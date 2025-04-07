//! ディスプレイモジュール
//! 
//! このモジュールはリモートデスクトップクライアントの表示機能を担当します。
//! 主にサーバーから受信した画像データのデコードと画面表示を行います。

mod decoder;
mod renderer;
mod command;


pub use decoder::{ImageDecoder, DecodedImage, DecodeError};
pub use renderer::{DisplayRenderer, RenderError};
pub use command::Command;

/// 画像フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG形式
    JPEG,
    /// PNG形式
    PNG,
    /// WebP形式
    WebP,
    /// AVIF形式
    AVIF,
}

/// 画像データ
#[derive(Debug, Clone)]
pub struct ImageData {
    /// 画像バイナリデータ
    pub data: Vec<u8>,
    /// 画像フォーマット
    pub format: ImageFormat,
    /// 画像の幅
    pub width: u32,
    /// 画像の高さ
    pub height: u32,
    /// タイムスタンプ（ミリ秒）
    pub timestamp: u64,
}