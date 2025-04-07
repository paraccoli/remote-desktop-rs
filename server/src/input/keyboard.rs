//! キーボード入力処理
//!
//! リモートデスクトップサーバーにおけるキーボード入力処理を実装します。

use super::{system::{SystemInput, SystemKey, SystemKeyModifier, SystemError}, InputError, KeyboardConfig};
use std::time::Duration;
use std::thread;

/// キーボード入力処理
pub struct KeyboardHandler {
    /// システム入力
    system: SystemInput,
    /// キーボード設定
    config: KeyboardConfig,
    /// キーマッピング
    key_map: KeyMapping,
}

impl KeyboardHandler {
    /// 新しいキーボードハンドラを作成
    pub fn new() -> Result<Self, InputError> {
        let system = SystemInput::new()
            .map_err(|e| InputError::SystemError(e.to_string()))?;
        
        Ok(Self {
            system,
            config: KeyboardConfig::default(),
            key_map: KeyMapping::new(),
        })
    }
    
    /// 設定を更新
    pub fn set_config(&mut self, config: KeyboardConfig) {
        self.config = config;
    }
    
    /// 設定を取得
    pub fn config(&self) -> &KeyboardConfig {
        &self.config
    }
    
    /// キーコードをマップ
    pub fn map_key_code(&self, key_code: u32) -> SystemKey {
        if self.config.key_mapping {
            self.key_map.map_key(key_code)
        } else {
            SystemKey(key_code)
        }
    }
    
    /// キーを押下
    pub fn key_down(&self, key: SystemKey, modifiers: &[SystemKeyModifier]) -> Result<(), InputError> {
        // 設定に応じてアクションをブロック
        if self.config.block_shortcuts && self.is_dangerous_shortcut(key, modifiers) {
            return Err(InputError::InvalidInput(format!("ブロックされたショートカット: {:?}", key)));
        }
        
        // 修飾キーを押下
        for modifier in modifiers {
            let modifier_key = self.modifier_to_key(*modifier);
            self.system.key_down(modifier_key)
                .map_err(|e| InputError::SystemError(e.to_string()))?;
        }
        
        // キーを押下
        self.system.key_down(key)
            .map_err(|e| InputError::SystemError(e.to_string()))?;
        
        Ok(())
    }
    
    /// キーを解放
    pub fn key_up(&self, key: SystemKey, modifiers: &[SystemKeyModifier]) -> Result<(), InputError> {
        // キーを解放
        self.system.key_up(key)
            .map_err(|e| InputError::SystemError(e.to_string()))?;
        
        // 修飾キーを解放
        for modifier in modifiers {
            let modifier_key = self.modifier_to_key(*modifier);
            self.system.key_up(modifier_key)
                .map_err(|e| InputError::SystemError(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// テキストを入力
    pub fn input_text(&self, text: &str) -> Result<(), InputError> {
        self.system.input_text(text)
            .map_err(|e| InputError::SystemError(e.to_string()))
    }
    
    /// キーコンビネーションを実行
    pub fn key_combo(&self, keys: &[SystemKey], modifiers: &[SystemKeyModifier]) -> Result<(), InputError> {
        // 設定に応じてアクションをブロック
        if self.config.block_shortcuts && keys.iter().any(|k| self.is_dangerous_shortcut(*k, modifiers)) {
            return Err(InputError::InvalidInput("ブロックされたショートカット".to_string()));
        }
        
        // 修飾キーを押下
        for modifier in modifiers {
            let modifier_key = self.modifier_to_key(*modifier);
            self.system.key_down(modifier_key)
                .map_err(|e| InputError::SystemError(e.to_string()))?;
        }
        
        // 各キーを押下して解放
        for key in keys {
            self.system.key_down(*key)
                .map_err(|e| InputError::SystemError(e.to_string()))?;
            thread::sleep(Duration::from_millis(10));
            self.system.key_up(*key)
                .map_err(|e| InputError::SystemError(e.to_string()))?;
            thread::sleep(Duration::from_millis(10));
        }
        
        // 修飾キーを解放
        for modifier in modifiers {
            let modifier_key = self.modifier_to_key(*modifier);
            self.system.key_up(modifier_key)
                .map_err(|e| InputError::SystemError(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// 危険なショートカットかどうかを判定
    fn is_dangerous_shortcut(&self, key: SystemKey, modifiers: &[SystemKeyModifier]) -> bool {
        let has_ctrl = modifiers.contains(&SystemKeyModifier::Control);
        let has_alt = modifiers.contains(&SystemKeyModifier::Alt);
        let has_shift = modifiers.contains(&SystemKeyModifier::Shift);
        let has_meta = modifiers.contains(&SystemKeyModifier::Meta);
        
        // Windowsキー + R
        if has_meta && key.0 == 0x52 {
            return true;
        }
        
        // Alt + F4
        if has_alt && key.0 == 0x73 {
            return true;
        }
        
        // Ctrl + Alt + Delete
        if has_ctrl && has_alt && key.0 == 0x2E {
            return true;
        }
        
        // その他の危険なショートカット
        false
    }
    
    /// 修飾子をキーに変換
    fn modifier_to_key(&self, modifier: SystemKeyModifier) -> SystemKey {
        match modifier {
            SystemKeyModifier::Shift => SystemKey(0x10),   // VK_SHIFT
            SystemKeyModifier::Control => SystemKey(0x11), // VK_CONTROL
            SystemKeyModifier::Alt => SystemKey(0x12),     // VK_MENU
            SystemKeyModifier::Meta => SystemKey(0x5B),    // VK_LWIN
            SystemKeyModifier::CapsLock => SystemKey(0x14), // VK_CAPITAL
            SystemKeyModifier::NumLock => SystemKey(0x90),  // VK_NUMLOCK
        }
    }
}

/// キーマッピング
struct KeyMapping {
    // キーコードマップ
    map: std::collections::HashMap<u32, u32>,
}

impl KeyMapping {
    /// 新しいキーマッピングを作成
    fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }
    
    /// キーを変換
    fn map_key(&self, key_code: u32) -> SystemKey {
        let mapped = self.map.get(&key_code).copied().unwrap_or(key_code);
        SystemKey(mapped)
    }
    
    /// キーマッピングを追加
    fn add_mapping(&mut self, from: u32, to: u32) {
        self.map.insert(from, to);
    }
}