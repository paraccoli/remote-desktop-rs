//! 画面レンダリングモジュール
//! 
//! デコードされた画像をGUIに表示するための機能を提供します。

use super::decoder::DecodedImage;
use egui::{Context, TextureHandle, TextureId, Color32, pos2, vec2, Rect, Pos2, Vec2};
use thiserror::Error;
use std::sync::{Arc, Mutex};

/// レンダリングエラー
#[derive(Error, Debug)]
pub enum RenderError {
    /// テクスチャ作成エラー
    #[error("テクスチャの作成に失敗しました: {0}")]
    TextureCreationFailed(String),
    
    /// 画像変換エラー
    #[error("画像変換に失敗しました: {0}")]
    ImageConversionFailed(String),
    
    /// その他のエラー
    #[error("レンダリング中に予期しないエラーが発生しました: {0}")]
    Other(String),
}

/// ディスプレイレンダラー
pub struct DisplayRenderer {
    /// eguiコンテキスト
    ctx: Context,
    /// 表示用テクスチャ
    texture: Option<TextureHandle>,
    /// 現在の画像サイズ
    image_size: Vec2,
    /// 画像の表示位置とサイズ
    display_rect: Option<Rect>,
    /// マウス座標変換用のスケール係数
    scale_factor: f32,
}

impl DisplayRenderer {
    /// 新しいディスプレイレンダラーを作成
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx,
            texture: None,
            image_size: vec2(0.0, 0.0),
            display_rect: None,
            scale_factor: 1.0,
        }
    }
    
    /// 画像を更新
    pub fn update_image(&mut self, data: Vec<u8>, width: u32, height: u32) -> Result<(), RenderError> {
        // RGBA形式の画像データに変換
        let size = [width as usize, height as usize];
        
        // 画像データをeguiのテクスチャデータに変換
        let color_image = self.convert_to_egui_image(&data, width, height)?;
        
        // テクスチャを更新または作成
        if let Some(texture) = &mut self.texture {
            texture.set(color_image);
        } else {
            self.texture = Some(self.ctx.load_texture(
                "remote_display",
                color_image,
                egui::TextureFilter::Linear
            ));
        }
        
        self.image_size = vec2(width as f32, height as f32);
        
        Ok(())
    }
    
    /// デコードされた画像を更新
    pub fn update_from_decoded(&mut self, decoded: &DecodedImage) -> Result<(), RenderError> {
        let image = &decoded.image;
        let width = image.width();
        let height = image.height();
        
        // RGBAに変換
        let rgba_image = image.to_rgba8();
        let pixels = rgba_image.into_raw();
        
        self.update_image(pixels, width, height)
    }
    
    /// 画像データをeguiの形式に変換
    fn convert_to_egui_image(&self, data: &[u8], width: u32, height: u32) -> Result<egui::ColorImage, RenderError> {
        // 入力がRGBA8形式であることを想定
        if data.len() != (width as usize * height as usize * 4) {
            return Err(RenderError::ImageConversionFailed(
                format!("Invalid data size: expected {} but got {}", width as usize * height as usize * 4, data.len())
            ));
        }
        
        let pixels: Vec<Color32> = data
            .chunks_exact(4)
            .map(|chunk| Color32::from_rgba_unmultiplied(chunk[0], chunk[1], chunk[2], chunk[3]))
            .collect();
        
        Ok(egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &pixels
        ))
    }
    
    /// テクスチャIDを取得
    pub fn get_texture(&self) -> Option<TextureId> {
        self.texture.as_ref().map(|t| t.id())
    }
    
    /// 画像サイズを取得
    pub fn get_image_size(&self) -> Vec2 {
        self.image_size
    }
    
    /// 表示領域を設定
    pub fn set_display_rect(&mut self, rect: Rect) {
        self.display_rect = Some(rect);
        
        // スケール係数を計算
        if self.image_size.x > 0.0 && self.image_size.y > 0.0 {
            let display_size = rect.size();
            self.scale_factor = (self.image_size.x / display_size.x)
                .max(self.image_size.y / display_size.y);
        }
    }
    
    /// 表示領域を取得
    pub fn get_display_rect(&self) -> Option<Rect> {
        self.display_rect
    }
    
    /// UI座標をスクリーン座標に変換
    pub fn convert_to_screen_pos(&self, ui_pos: Pos2, origin: Pos2, display_size: Vec2) -> Pos2 {
        if self.image_size.x == 0.0 || self.image_size.y == 0.0 {
            return ui_pos;
        }
        
        // 表示領域内の相対位置を計算
        let relative_pos = (ui_pos - origin) / display_size;
        
        // スクリーン座標に変換
        pos2(
            relative_pos.x * self.image_size.x,
            relative_pos.y * self.image_size.y
        )
    }
}