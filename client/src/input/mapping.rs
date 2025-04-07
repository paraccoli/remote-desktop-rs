//! 入力マッピングモジュール
//!
//! プラットフォーム固有のキーコードを共通のプロトコル形式にマッピングします。

use super::MouseButton;
use egui::Key;
use std::collections::HashMap;

/// キーボードマッピング
#[derive(Debug, Clone)]
pub struct KeyMapping {
    /// キーマップ
    key_map: HashMap<Key, Key>,
    /// 文字からキーへのマップ
    char_map: HashMap<char, Key>,
}

impl KeyMapping {
    /// 新しいキーマッピングを作成
    pub fn new() -> Self {
        let mut key_map = HashMap::new();
        let mut char_map = HashMap::new();

        // 基本的なキーは変換なしで渡す
        // 特定のプラットフォーム固有のキーがある場合はここでマッピング

        // 文字とキーのマッピング
        for c in 'a'..='z' {
            let key = match c {
                'a' => Key::A,
                'b' => Key::B,
                'c' => Key::C,
                'd' => Key::D,
                'e' => Key::E,
                'f' => Key::F,
                'g' => Key::G,
                'h' => Key::H,
                'i' => Key::I,
                'j' => Key::J,
                'k' => Key::K,
                'l' => Key::L,
                'm' => Key::M,
                'n' => Key::N,
                'o' => Key::O,
                'p' => Key::P,
                'q' => Key::Q,
                'r' => Key::R,
                's' => Key::S,
                't' => Key::T,
                'u' => Key::U,
                'v' => Key::V,
                'w' => Key::W,
                'x' => Key::X,
                'y' => Key::Y,
                'z' => Key::Z,
                _ => continue,
            };
            char_map.insert(c, key);
            char_map.insert(c.to_ascii_uppercase(), key);
        }

        // 数字
        for (i, c) in ('0'..='9').enumerate() {
            let key = match i {
                0 => Key::Num0,
                1 => Key::Num1,
                2 => Key::Num2,
                3 => Key::Num3,
                4 => Key::Num4,
                5 => Key::Num5,
                6 => Key::Num6,
                7 => Key::Num7,
                8 => Key::Num8,
                9 => Key::Num9,
                _ => continue,
            };
            char_map.insert(c, key);
        }

        // 特殊文字
        let special_chars = [
            (' ', Key::Space),
            ('\t', Key::Tab),
            ('\n', Key::Enter),
            ('\r', Key::Enter),
            ('-', Key::Minus),
            ('=', Key::Equals),
            ('[', Key::LeftBracket),
            (']', Key::RightBracket),
            ('\\', Key::Backslash),
            (';', Key::Semicolon),
            ('\'', Key::Quote),
            (',', Key::Comma),
            ('.', Key::Period),
            ('/', Key::Slash),
        ];

        for (c, key) in special_chars {
            char_map.insert(c, key);
        }

        Self { key_map, char_map }
    }

    /// キーを変換
    pub fn map_key(&self, key: Key) -> Key {
        *self.key_map.get(&key).unwrap_or(&key)
    }

    /// 文字からキーへの変換
    pub fn char_to_key(&self, c: char) -> Option<Key> {
        self.char_map.get(&c).copied()
    }

    /// キーマッピングの追加
    pub fn add_mapping(&mut self, from: Key, to: Key) {
        self.key_map.insert(from, to);
    }

    /// 文字マッピングの追加
    pub fn add_char_mapping(&mut self, c: char, key: Key) {
        self.char_map.insert(c, key);
    }
}

impl Default for KeyMapping {
    fn default() -> Self {
        Self::new()
    }
}

/// マウスマッピング
#[derive(Debug, Clone)]
pub struct MouseMapping {
    /// ボタンマップ
    button_map: HashMap<u8, MouseButton>,
}

impl MouseMapping {
    /// 新しいマウスマッピングを作成
    pub fn new() -> Self {
        let mut button_map = HashMap::new();

        // 標準的なマウスボタンマッピング
        button_map.insert(1, MouseButton::Left);
        button_map.insert(2, MouseButton::Middle);
        button_map.insert(3, MouseButton::Right);
        button_map.insert(4, MouseButton::X1);
        button_map.insert(5, MouseButton::X2);

        Self { button_map }
    }

    /// ボタン番号からマウスボタンへの変換
    pub fn map_button(&self, button: u8) -> Option<MouseButton> {
        self.button_map.get(&button).copied()
    }

    /// ボタンマッピングの追加
    pub fn add_mapping(&mut self, button_number: u8, button: MouseButton) {
        self.button_map.insert(button_number, button);
    }
}

impl Default for MouseMapping {
    fn default() -> Self {
        Self::new()
    }
}