//! 入力処理モジュール
//!
//! リモートデスクトップサーバーで受信した入力コマンドを処理し、
//! システムの入力イベントに変換して実行します。

pub mod system;
pub mod keyboard;
pub mod mouse;
pub mod mapping;

use crate::error::ServerError;
use remote_desktop_rs_common::protocol::{Command, KeyModifier, MouseButton};
use thiserror::Error;

/// 入力エラー
#[derive(Error, Debug)]
pub enum InputError {
    /// キーボード入力エラー
    #[error("キーボード入力エラー: {0}")]
    KeyboardError(String),
    
    /// マウス入力エラー
    #[error("マウス入力エラー: {0}")]
    MouseError(String),
    
    /// 不正な入力
    #[error("不正な入力: {0}")]
    InvalidInput(String),
    
    /// システム入力エラー
    #[error("システム入力エラー: {0}")]
    SystemError(String),
    
    /// その他のエラー
    #[error("入力エラー: {0}")]
    Other(String),
}

/// 入力ハンドラ
pub struct InputHandler {
    /// キーボードハンドラ
    keyboard: keyboard::KeyboardHandler,
    /// マウスハンドラ
    mouse: mouse::MouseHandler,
    /// 入力設定
    config: InputConfig,
}

impl InputHandler {
    /// 新しい入力ハンドラを作成
    pub fn new() -> Result<Self, InputError> {
        let keyboard = keyboard::KeyboardHandler::new()?;
        let mouse = mouse::MouseHandler::new()?;
        let config = InputConfig::default();
        
        Ok(Self {
            keyboard,
            mouse,
            config,
        })
    }
    
    /// 設定を更新
    pub fn set_config(&mut self, config: InputConfig) {
        self.config = config;
        self.keyboard.set_config(config.keyboard);
        self.mouse.set_config(config.mouse);
    }
    
    /// 設定を取得
    pub fn config(&self) -> &InputConfig {
        &self.config
    }
    
    /// コマンドを処理
    pub fn handle_command(&mut self, command: &Command) -> Result<(), InputError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        match command {
            Command::MouseMove { x, y } => {
                if self.config.mouse.enabled {
                    self.mouse.move_to(*x, *y)?;
                }
            },
            Command::MouseClick { button, double } => {
                if self.config.mouse.enabled {
                    let button = self.map_mouse_button(*button);
                    if *double {
                        self.mouse.double_click(button)?;
                    } else {
                        self.mouse.click(button)?;
                    }
                }
            },
            Command::MouseDown { button } => {
                if self.config.mouse.enabled {
                    let button = self.map_mouse_button(*button);
                    self.mouse.button_down(button)?;
                }
            },
            Command::MouseUp { button } => {
                if self.config.mouse.enabled {
                    let button = self.map_mouse_button(*button);
                    self.mouse.button_up(button)?;
                }
            },
            Command::MouseScroll { delta_x, delta_y } => {
                if self.config.mouse.enabled {
                    self.mouse.scroll(*delta_x, *delta_y)?;
                }
            },
            Command::KeyDown { key_code, modifiers } => {
                if self.config.keyboard.enabled {
                    let mapped_key = self.keyboard.map_key_code(*key_code);
                    let mapped_modifiers = modifiers.iter()
                        .map(|m| self.map_key_modifier(*m))
                        .collect();
                    self.keyboard.key_down(mapped_key, &mapped_modifiers)?;
                }
            },
            Command::KeyUp { key_code, modifiers } => {
                if self.config.keyboard.enabled {
                    let mapped_key = self.keyboard.map_key_code(*key_code);
                    let mapped_modifiers = modifiers.iter()
                        .map(|m| self.map_key_modifier(*m))
                        .collect();
                    self.keyboard.key_up(mapped_key, &mapped_modifiers)?;
                }
            },
            Command::TextInput { text } => {
                if self.config.keyboard.enabled {
                    self.keyboard.input_text(text)?;
                }
            },
            Command::KeyCombo { key_codes, modifiers } => {
                if self.config.keyboard.enabled {
                    let mapped_keys = key_codes.iter()
                        .map(|k| self.keyboard.map_key_code(*k))
                        .collect();
                    let mapped_modifiers = modifiers.iter()
                        .map(|m| self.map_key_modifier(*m))
                        .collect();
                    self.keyboard.key_combo(&mapped_keys, &mapped_modifiers)?;
                }
            },
            _ => {
                // 他のコマンドは入力処理の対象外
            },
        }
        
        Ok(())
    }
    
    /// マウスボタンをマップ
    fn map_mouse_button(&self, button: MouseButton) -> system::SystemMouseButton {
        match button {
            MouseButton::Left => system::SystemMouseButton::Left,
            MouseButton::Right => system::SystemMouseButton::Right,
            MouseButton::Middle => system::SystemMouseButton::Middle,
            MouseButton::Back => system::SystemMouseButton::Back,
            MouseButton::Forward => system::SystemMouseButton::Forward,
        }
    }
    
    /// キー修飾子をマップ
    fn map_key_modifier(&self, modifier: KeyModifier) -> system::SystemKeyModifier {
        match modifier {
            KeyModifier::Shift => system::SystemKeyModifier::Shift,
            KeyModifier::Control => system::SystemKeyModifier::Control,
            KeyModifier::Alt => system::SystemKeyModifier::Alt,
            KeyModifier::Meta => system::SystemKeyModifier::Meta,
            KeyModifier::CapsLock => system::SystemKeyModifier::CapsLock,
            KeyModifier::NumLock => system::SystemKeyModifier::NumLock,
        }
    }
}

/// 入力設定
#[derive(Debug, Clone, Copy)]
pub struct InputConfig {
    /// 入力処理を有効化
    pub enabled: bool,
    /// キーボード設定
    pub keyboard: KeyboardConfig,
    /// マウス設定
    pub mouse: MouseConfig,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            keyboard: KeyboardConfig::default(),
            mouse: MouseConfig::default(),
        }
    }
}

/// キーボード設定
#[derive(Debug, Clone, Copy)]
pub struct KeyboardConfig {
    /// キーボード入力を有効化
    pub enabled: bool,
    /// キー変換を有効化
    pub key_mapping: bool,
    /// ショートカットブロックを有効化
    pub block_shortcuts: bool,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_mapping: true,
            block_shortcuts: true,
        }
    }
}

/// マウス設定
#[derive(Debug, Clone, Copy)]
pub struct MouseConfig {
    /// マウス入力を有効化
    pub enabled: bool,
    /// 相対座標を使用
    pub use_relative: bool,
    /// スクロール速度
    pub scroll_speed: f32,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_relative: false,
            scroll_speed: 1.0,
        }
    }
}