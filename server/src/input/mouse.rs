//! マウス入力処理
//!
//! リモートデスクトップサーバーにおけるマウス入力処理を実装します。

use super::{system::{SystemInput, SystemMouseButton, SystemError}, InputError, MouseConfig};
use std::time::Duration;
use std::thread;

/// マウス入力処理
pub struct MouseHandler {
    /// システム入力
    system: SystemInput,
    /// マウス設定
    config: MouseConfig,
    /// 前回の座標
    last_position: Option<(i32, i32)>,
}

impl MouseHandler {
    /// 新しいマウスハンドラを作成
    pub fn new() -> Result<Self, InputError> {
        let system = SystemInput::new()
            .map_err(|e| InputError::SystemError(e.to_string()))?;
        
        Ok(Self {
            system,
            config: MouseConfig::default(),
            last_position: None,
        })
    }
    
    /// 設定を更新
    pub fn set_config(&mut self, config: MouseConfig) {
        self.config = config;
    }
    
    /// 設定を取得
    pub fn config(&self) -> &MouseConfig {
        &self.config
    }
    
    /// マウスを移動
    pub fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError> {
        if self.config.use_relative && self.last_position.is_some() {
            let (last_x, last_y) = self.last_position.unwrap();
            let dx = x - last_x;
            let dy = y - last_y;
            
            if dx != 0 || dy != 0 {
                self.system.mouse_move_relative(dx, dy)
                    .map_err(|e| InputError::SystemError(e.to_string()))?;
            }
        } else {
            self.system.mouse_move(x, y)
                .map_err(|e| InputError::SystemError(e.to_string()))?;
        }
        
        self.last_position = Some((x, y));
        
        Ok(())
    }
    
    /// マウスを相対的に移動
    pub fn move_relative(&mut self, dx: i32, dy: i32) -> Result<(), InputError> {
        self.system.mouse_move_relative(dx, dy)
            .map_err(|e| InputError::SystemError(e.to_string()))?;
        
        // 現在位置を更新
        if let Ok(position) = self.system.get_mouse_position() {
            self.last_position = Some(position);
        }
        
        Ok(())
    }
    
    /// マウスボタン押下
    pub fn button_down(&self, button: SystemMouseButton) -> Result<(), InputError> {
        self.system.mouse_down(button)
            .map_err(|e| InputError::SystemError(e.to_string()))
    }
    
    /// マウスボタン解放
    pub fn button_up(&self, button: SystemMouseButton) -> Result<(), InputError> {
        self.system.mouse_up(button)
            .map_err(|e| InputError::SystemError(e.to_string()))
    }
    
    /// クリック（押下＋解放）
    pub fn click(&self, button: SystemMouseButton) -> Result<(), InputError> {
        self.button_down(button)?;
        thread::sleep(Duration::from_millis(10));
        self.button_up(button)?;
        
        Ok(())
    }
    
    /// ダブルクリック
    pub fn double_click(&self, button: SystemMouseButton) -> Result<(), InputError> {
        self.click(button)?;
        thread::sleep(Duration::from_millis(50));
        self.click(button)?;
        
        Ok(())
    }
    
    /// スクロール
    pub fn scroll(&self, delta_x: i32, delta_y: i32) -> Result<(), InputError> {
        let scale = self.config.scroll_speed;
        let dx = (delta_x as f32 * scale) as i32;
        let dy = (delta_y as f32 * scale) as i32;
        
        self.system.mouse_scroll(dx, dy)
            .map_err(|e| InputError::SystemError(e.to_string()))
    }
    
    /// 現在位置を取得
    pub fn get_position(&self) -> Result<(i32, i32), InputError> {
        self.system.get_mouse_position()
            .map_err(|e| InputError::SystemError(e.to_string()))
    }
    
    /// スクリーンサイズを取得
    pub fn get_screen_size(&self) -> Result<(u32, u32), InputError> {
        self.system.get_screen_size()
            .map_err(|e| InputError::SystemError(e.to_string()))
    }
}