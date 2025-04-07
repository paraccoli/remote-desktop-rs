//! 画像エンコーダモジュール
//!
//! キャプチャした画像を効率的にエンコードするための機能を提供します。

use super::{CapturedImage, EncodedImage, ImageFormat};
use crate::error::ServerError;

use image::{DynamicImage, ImageOutputFormat};
use thiserror::Error;
use std::io::{Cursor, BufWriter};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// エンコードエラー
#[derive(Error, Debug)]
pub enum EncoderError {
    /// 入出力エラー
    #[error("入出力エラー: {0}")]
    IoError(#[from] std::io::Error),
    
    /// エンコードエラー
    #[error("エンコードエラー: {0}")]
    EncodeError(String),
    
    /// フォーマットエラー
    #[error("フォーマットエラー: {0}")]
    FormatError(String),
    
    /// その他のエラー
    #[error("エンコードエラー: {0}")]
    Other(String),
}

/// エンコーダ設定
#[derive(Debug, Clone, Copy)]
pub struct EncoderConfig {
    /// 画像フォーマット
    pub format: ImageFormat,
    /// JPEG/WebP品質（0-100）
    pub quality: u8,
    /// リサイズする最大幅
    pub max_width: Option<u32>,
    /// リサイズする最大高さ
    pub max_height: Option<u32>,
    /// アルファチャンネルを保持
    pub preserve_alpha: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            format: ImageFormat::JPEG,
            quality: 75,
            max_width: None,
            max_height: None,
            preserve_alpha: false,
        }
    }
}

/// 画像エンコーダ
pub struct ImageEncoder {
    /// エンコーダ設定
    config: EncoderConfig,
}

impl ImageEncoder {
    /// 新しい画像エンコーダを作成
    pub fn new(config: EncoderConfig) -> Self {
        Self { config }
    }
    
    /// 設定を更新
    pub fn set_config(&mut self, config: EncoderConfig) {
        self.config = config;
    }
    
    /// 設定を取得
    pub fn config(&self) -> &EncoderConfig {
        &self.config
    }
    
    /// キャプチャした画像をエンコード
    pub fn encode(&self, image: &CapturedImage) -> Result<EncodedImage, EncoderError> {
        // 必要に応じてリサイズ
        let processed_image = self.preprocess_image(image)?;
        
        // 元画像のサイズを保存（圧縮率計算用）
        let original_size = processed_image.width() * processed_image.height() * 4; // RGBA 4バイト/ピクセル
        
        // エンコード
        let mut buffer = Cursor::new(Vec::new());
        let start_time = Instant::now();
        
        match self.config.format {
            ImageFormat::JPEG => {
                let quality = self.config.quality.clamp(1, 100);
                processed_image.write_to(&mut buffer, ImageOutputFormat::Jpeg(quality))?;
            },
            ImageFormat::PNG => {
                processed_image.write_to(&mut buffer, ImageOutputFormat::Png)?;
            },
            ImageFormat::WebP => {
                #[cfg(feature = "webp")]
                {
                    let quality = self.config.quality.clamp(1, 100);
                    let encoder = webp::Encoder::from_image(&processed_image)
                        .map_err(|e| EncoderError::EncodeError(format!("WebP初期化エラー: {}", e)))?;
                    
                    let webp_data = if self.config.preserve_alpha {
                        encoder.encode(quality as f32)
                    } else {
                        encoder.encode_lossless()
                    };
                    
                    buffer.write_all(&webp_data)
                        .map_err(|e| EncoderError::IoError(e))?;
                }
                
                #[cfg(not(feature = "webp"))]
                {
                    return Err(EncoderError::FormatError("WebPサポートが有効になっていません".to_string()));
                }
            },
            ImageFormat::AVIF => {
                #[cfg(feature = "avif")]
                {
                    // AVIFエンコード実装
                    // 注: 現在RustではAVIFエンコード用のよく整備されたクレートがまだ少ないため、
                    // ここではプレースホルダーとしています。
                    
                    // 実際のプロジェクトでは libavif に対する FFI を実装するか、
                    // rav1e などを使用したカスタムエンコードが必要になります。
                    
                    return Err(EncoderError::EncodeError("AVIF encoding not yet implemented".to_string()));
                }
                
                #[cfg(not(feature = "avif"))]
                {
                    return Err(EncoderError::FormatError("AVIFサポートが有効になっていません".to_string()));
                }
            },
        }
        
        let encode_time = start_time.elapsed();
        log::debug!("画像エンコード時間: {:?}", encode_time);
        
        // エンコードされたデータを取得
        let data = buffer.into_inner();
        let compressed_size = data.len();
        
        // 圧縮率を計算 (1.0 = 非圧縮、0.5 = 50%圧縮)
        let compression_ratio = if original_size > 0 {
            (compressed_size as f32 / original_size as f32) * 100.0
        } else {
            100.0
        };
        
        // タイムスタンプを取得（ミリ秒単位のUNIXタイムスタンプ）
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(EncodedImage {
            data,
            format: self.config.format,
            width: processed_image.width(),
            height: processed_image.height(),
            timestamp,
            monitor_index: image.monitor_index,
            encode_time_ms: encode_time.as_millis() as u64,
        })
    }
    
    /// 画像の前処理（リサイズなど）
    fn preprocess_image(&self, image: &CapturedImage) -> Result<DynamicImage, EncoderError> {
        let mut processed = image.image.clone();
        
        // リサイズが必要か確認
        if let (Some(max_width), Some(max_height)) = (self.config.max_width, self.config.max_height) {
            let width = processed.width();
            let height = processed.height();
            
            if width > max_width || height > max_height {
                // アスペクト比を維持するために、どちらかの次元に合わせてスケーリング
                let width_ratio = max_width as f32 / width as f32;
                let height_ratio = max_height as f32 / height as f32;
                
                let ratio = width_ratio.min(height_ratio);
                let new_width = (width as f32 * ratio) as u32;
                let new_height = (height as f32 * ratio) as u32;
                
                processed = processed.resize_exact(
                    new_width,
                    new_height,
                    image::imageops::FilterType::Lanczos3, // 高品質なリサイズ
                );
            }
        } else if let Some(max_width) = self.config.max_width {
            let width = processed.width();
            if width > max_width {
                let ratio = max_width as f32 / width as f32;
                let new_height = (processed.height() as f32 * ratio) as u32;
                processed = processed.resize(
                    max_width,
                    new_height,
                    image::imageops::FilterType::Lanczos3,
                );
            }
        } else if let Some(max_height) = self.config.max_height {
            let height = processed.height();
            if height > max_height {
                let ratio = max_height as f32 / height as f32;
                let new_width = (processed.width() as f32 * ratio) as u32;
                processed = processed.resize(
                    new_width,
                    max_height,
                    image::imageops::FilterType::Lanczos3,
                );
            }
        }
        
        // アルファチャンネルの処理
        if !self.config.preserve_alpha && matches!(self.config.format, ImageFormat::JPEG) {
            // JPEGはアルファをサポートしないので、背景で合成
            processed = DynamicImage::ImageRgb8(processed.to_rgb8());
        }
        
        Ok(processed)
    }
    
    /// エンコード方式を変更
    pub fn set_format(&mut self, format: ImageFormat) {
        self.config.format = format;
    }
    
    /// 画質を設定
    pub fn set_quality(&mut self, quality: u8) {
        self.config.quality = quality.clamp(1, 100);
    }
    
    /// 最大サイズを設定
    pub fn set_max_size(&mut self, width: Option<u32>, height: Option<u32>) {
        self.config.max_width = width;
        self.config.max_height = height;
    }
}

impl Default for ImageEncoder {
    fn default() -> Self {
        Self::new(EncoderConfig::default())
    }
}