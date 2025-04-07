//! キャプチャモジュール
//!
//! スクリーンショットのキャプチャと画像処理を担当します。

pub mod screenshot;
pub mod encoder;
pub mod monitor;
pub mod diff;

// 主要なコンポーネントを再エクスポート
pub use screenshot::{ScreenCapture, CaptureError};
pub use encoder::{ImageEncoder, EncoderConfig, EncoderError};
pub use monitor::{Monitor, MonitorInfo};
pub use diff::{DiffCalculator, DiffResult, Rectangle};

use image::{DynamicImage, ImageBuffer, Rgba};
use std::time::Instant;

/// キャプチャーイメージ
#[derive(Debug, Clone)]
pub struct CapturedImage {
    /// 画像データ
    pub image: DynamicImage,
    /// キャプチャ時のタイムスタンプ
    pub timestamp: Instant,
    /// 元の解像度（幅）
    pub original_width: u32,
    /// 元の解像度（高さ）
    pub original_height: u32,
    /// モニターインデックス
    pub monitor_index: usize,
}

impl CapturedImage {
    /// 新しいキャプチャーイメージを作成
    pub fn new(image: DynamicImage, monitor_index: usize) -> Self {
        let original_width = image.width();
        let original_height = image.height();
        
        Self {
            image,
            timestamp: Instant::now(),
            original_width,
            original_height,
            monitor_index,
        }
    }
    
    /// リサイズされたイメージを生成
    pub fn resize(&self, width: u32, height: u32) -> Self {
        if width == self.original_width && height == self.original_height {
            return self.clone();
        }
        
        let resized = self.image.resize(width, height, image::imageops::FilterType::Lanczos3);
        
        Self {
            image: resized,
            timestamp: self.timestamp,
            original_width: self.original_width,
            original_height: self.original_height,
            monitor_index: self.monitor_index,
        }
    }
    
    /// 領域を切り出す
    pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Option<Self> {
        let img_width = self.image.width();
        let img_height = self.image.height();
        
        if x >= img_width || y >= img_height {
            return None;
        }
        
        let crop_width = if x + width > img_width { img_width - x } else { width };
        let crop_height = if y + height > img_height { img_height - y } else { height };
        
        if crop_width == 0 || crop_height == 0 {
            return None;
        }
        
        Some(Self {
            image: self.image.crop_imm(x, y, crop_width, crop_height),
            timestamp: self.timestamp,
            original_width: self.original_width,
            original_height: self.original_height,
            monitor_index: self.monitor_index,
        })
    }
    
    /// 画像のサイズを取得
    pub fn size(&self) -> (u32, u32) {
        (self.image.width(), self.image.height())
    }
}

/// エンコード済み画像
#[derive(Debug, Clone)]
pub struct EncodedImage {
    /// エンコードされたデータ
    pub data: Vec<u8>,
    /// エンコードフォーマット
    pub format: ImageFormat,
    /// 幅
    pub width: u32,
    /// 高さ
    pub height: u32,
    /// エンコード時のタイムスタンプ（ミリ秒）
    pub timestamp: u64,
    /// 元の画像サイズ
    pub original_size: usize,
    /// 圧縮率（%）
    pub compression_ratio: f32,
}

/// 画像フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG
    JPEG,
    /// PNG
    PNG,
    /// WebP
    WebP,
    /// AVIF
    AVIF,
}

impl ImageFormat {
    /// 文字列からフォーマットを取得
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "jpeg" | "jpg" => Some(ImageFormat::JPEG),
            "png" => Some(ImageFormat::PNG),
            "webp" => Some(ImageFormat::WebP),
            "avif" => Some(ImageFormat::AVIF),
            _ => None,
        }
    }
    
    /// MIME タイプを取得
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::JPEG => "image/jpeg",
            ImageFormat::PNG => "image/png",
            ImageFormat::WebP => "image/webp",
            ImageFormat::AVIF => "image/avif",
        }
    }
    
    /// ファイル拡張子を取得
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::JPEG => "jpg",
            ImageFormat::PNG => "png",
            ImageFormat::WebP => "webp",
            ImageFormat::AVIF => "avif",
        }
    }
}