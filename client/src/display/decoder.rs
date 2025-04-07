//! 画像デコードモジュール
//! 
//! サーバーから受信した画像データをデコードする機能を提供します。

use super::ImageFormat;
use image::{DynamicImage, ImageError};
use thiserror::Error;
use std::io::Cursor;

/// デコードエラー
#[derive(Error, Debug)]
pub enum DecodeError {
    /// 画像フォーマットエラー
    #[error("不正な画像フォーマット: {0}")]
    InvalidFormat(String),
    
    /// デコードエラー
    #[error("画像のデコードに失敗しました: {0}")]
    DecodeFailure(#[from] ImageError),
    
    /// その他のエラー
    #[error("デコード中に予期しないエラーが発生しました: {0}")]
    Other(String),
}

/// デコードされた画像
#[derive(Debug, Clone)]
pub struct DecodedImage {
    /// 画像データ
    pub image: DynamicImage,
    /// 元の画像の幅
    pub original_width: u32,
    /// 元の画像の高さ
    pub original_height: u32,
    /// タイムスタンプ（ミリ秒）
    pub timestamp: u64,
}

/// 画像デコーダ
pub struct ImageDecoder {
    /// 最後にデコードした画像
    last_decoded: Option<DecodedImage>,
}

impl ImageDecoder {
    /// 新しい画像デコーダを作成
    pub fn new() -> Self {
        Self {
            last_decoded: None,
        }
    }
    
    /// 画像データをデコード
    pub fn decode(&mut self, data: &[u8], format: ImageFormat, width: u32, height: u32, timestamp: u64) -> Result<DecodedImage, DecodeError> {
        let image = match format {
            ImageFormat::JPEG => {
                image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)
                    .map_err(DecodeError::DecodeFailure)?
            },
            ImageFormat::PNG => {
                image::load_from_memory_with_format(data, image::ImageFormat::Png)
                    .map_err(DecodeError::DecodeFailure)?
            },
            ImageFormat::WebP => {
                // WebPはcrate 'image'では直接サポートされていないため、webp crateを使用
                #[cfg(feature = "webp")]
                {
                    let decoder = webp::Decoder::new(data);
                    let webp_image = decoder.decode()
                        .ok_or_else(|| DecodeError::InvalidFormat("WebP decoding failed".to_string()))?;
                    
                    let rgba = webp_image.to_rgba();
                    let width = webp_image.width();
                    let height = webp_image.height();
                    
                    DynamicImage::ImageRgba8(
                        image::RgbaImage::from_raw(width, height, rgba.to_vec())
                            .ok_or_else(|| DecodeError::Other("Failed to create image from WebP data".to_string()))?
                    )
                }
                
                #[cfg(not(feature = "webp"))]
                {
                    return Err(DecodeError::InvalidFormat("WebP format is not supported in this build".to_string()));
                }
            },
            ImageFormat::AVIF => {
                // AVIFもcrate 'image'では直接サポートされていないため、libavif-bindingsなどの外部crateが必要
                #[cfg(feature = "avif")]
                {
                    // AVIF対応時に実装
                    unimplemented!("AVIF decoding not yet implemented");
                }
                
                #[cfg(not(feature = "avif"))]
                {
                    return Err(DecodeError::InvalidFormat("AVIF format is not supported in this build".to_string()));
                }
            },
        };
        
        let decoded = DecodedImage {
            image,
            original_width: width,
            original_height: height,
            timestamp,
        };
        
        self.last_decoded = Some(decoded.clone());
        Ok(decoded)
    }
    
    /// Base64エンコードされた画像データをデコード
    pub fn decode_base64(&mut self, base64_data: &str, format: ImageFormat, width: u32, height: u32, timestamp: u64) -> Result<DecodedImage, DecodeError> {
        let data = base64::decode(base64_data)
            .map_err(|e| DecodeError::Other(format!("Base64 decoding failed: {}", e)))?;
        
        self.decode(&data, format, width, height, timestamp)
    }
    
    /// 最後にデコードした画像を取得
    pub fn last_image(&self) -> Option<&DecodedImage> {
        self.last_decoded.as_ref()
    }
}

impl Default for ImageDecoder {
    fn default() -> Self {
        Self::new()
    }
}