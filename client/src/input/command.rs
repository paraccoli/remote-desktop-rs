//! 入力コマンドモジュール
//!
//! 入力イベントから生成されるリモートサーバー向けのコマンドを定義します。

use super::{MouseButton, Modifier};
use egui::Key;
use serde::{Deserialize, Serialize};

/// リモートサーバーに送信されるコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// マウス移動
    MouseMove {
        /// X座標
        x: i32,
        /// Y座標
        y: i32,
    },
    
    /// マウスクリック
    MouseClick {
        /// ボタン
        button: MouseButton,
        /// ダブルクリックかどうか
        double: bool,
    },
    
    /// マウスボタン押下
    MouseDown {
        /// ボタン
        button: MouseButton,
    },
    
    /// マウスボタン解放
    MouseUp {
        /// ボタン
        button: MouseButton,
    },
    
    /// マウスホイールスクロール
    MouseScroll {
        /// X軸スクロール量
        delta_x: i32,
        /// Y軸スクロール量
        delta_y: i32,
    },
    
    /// キー押下
    KeyDown {
        /// キー
        key: Key,
    },
    
    /// キー解放
    KeyUp {
        /// キー
        key: Key,
    },
    
    /// キー押下と解放（単発）
    KeyPress {
        /// キー
        key: Key,
    },
    
    /// 修飾キーとの組み合わせ
    KeyCombo {
        /// キーの組み合わせ
        keys: Vec<Key>,
    },
    
    /// 複数キーの連続入力
    KeySequence {
        /// キーシーケンス
        keys: Vec<Key>,
    },
    
    /// 画質設定
    SetQuality {
        /// 画質値（1～100）
        quality: u8,
    },
    
    /// スクリーンショット要求
    RequestScreenshot {
        /// 画質（オプション）
        quality: Option<u8>,
        /// 幅（オプション）
        width: Option<u32>,
        /// 高さ（オプション）
        height: Option<u32>,
    },
    
    /// アプリケーション実行
    RunApplication {
        /// コマンド
        command: String,
    },
    
    /// 切断
    Disconnect,
}

impl Command {
    /// コマンドをサーバープロトコル文字列に変換
    pub fn to_protocol_string(&self) -> String {
        match self {
            Command::MouseMove { x, y } => {
                format!("MOUSE_MOVE {} {}", x, y)
            },
            Command::MouseClick { button, double } => {
                if *double {
                    format!("MOUSE_DOUBLE_CLICK {}", button.as_str())
                } else {
                    format!("MOUSE_CLICK {}", button.as_str())
                }
            },
            Command::MouseDown { button } => {
                format!("MOUSE_DOWN {}", button.as_str())
            },
            Command::MouseUp { button } => {
                format!("MOUSE_UP {}", button.as_str())
            },
            Command::MouseScroll { delta_x, delta_y } => {
                let direction = if *delta_y < 0 { "down" } else { "up" };
                let amount = delta_y.abs();
                format!("MOUSE_SCROLL {} {}", direction, amount)
            },
            Command::KeyDown { key } => {
                format!("KEY_DOWN {}", self.key_to_string(key))
            },
            Command::KeyUp { key } => {
                format!("KEY_UP {}", self.key_to_string(key))
            },
            Command::KeyPress { key } => {
                format!("KEY_PRESS {}", self.key_to_string(key))
            },
            Command::KeyCombo { keys } => {
                let key_str = keys.iter()
                    .map(|k| self.key_to_string(k))
                    .collect::<Vec<_>>()
                    .join("+");
                format!("KEY_COMBO {}", key_str)
            },
            Command::KeySequence { keys } => {
                let key_str = keys.iter()
                    .map(|k| self.key_to_string(k))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("KEY_SEQUENCE {}", key_str)
            },
            Command::SetQuality { quality } => {
                format!("SET_QUALITY {}", quality)
            },
            Command::RequestScreenshot { quality, width, height } => {
                let mut cmd = String::from("SCREENSHOT");
                if let Some(q) = quality {
                    cmd.push_str(&format!(" {}", q));
                }
                if let Some(w) = width {
                    cmd.push_str(&format!(" width={}", w));
                }
                if let Some(h) = height {
                    cmd.push_str(&format!(" height={}", h));
                }
                cmd
            },
            Command::RunApplication { command } => {
                format!("RUN {}", command)
            },
            Command::Disconnect => {
                String::from("DISCONNECT")
            },
        }
    }
    
    /// キーを文字列に変換
    fn key_to_string(&self, key: &Key) -> String {
        match key {
            Key::Space => "space".to_string(),
            Key::Enter => "return".to_string(),
            Key::Tab => "tab".to_string(),
            Key::Backspace => "backspace".to_string(),
            Key::Escape => "escape".to_string(),
            Key::A => "a".to_string(),
            Key::B => "b".to_string(),
            Key::C => "c".to_string(),
            Key::D => "d".to_string(),
            Key::E => "e".to_string(),
            Key::F => "f".to_string(),
            Key::G => "g".to_string(),
            Key::H => "h".to_string(),
            Key::I => "i".to_string(),
            Key::J => "j".to_string(),
            Key::K => "k".to_string(),
            Key::L => "l".to_string(),
            Key::M => "m".to_string(),
            Key::N => "n".to_string(),
            Key::O => "o".to_string(),
            Key::P => "p".to_string(),
            Key::Q => "q".to_string(),
            Key::R => "r".to_string(),
            Key::S => "s".to_string(),
            Key::T => "t".to_string(),
            Key::U => "u".to_string(),
            Key::V => "v".to_string(),
            Key::W => "w".to_string(),
            Key::X => "x".to_string(),
            Key::Y => "y".to_string(),
            Key::Z => "z".to_string(),
            Key::Num0 => "0".to_string(),
            Key::Num1 => "1".to_string(),
            Key::Num2 => "2".to_string(),
            Key::Num3 => "3".to_string(),
            Key::Num4 => "4".to_string(),
            Key::Num5 => "5".to_string(),
            Key::Num6 => "6".to_string(),
            Key::Num7 => "7".to_string(),
            Key::Num8 => "8".to_string(),
            Key::Num9 => "9".to_string(),
            Key::F1 => "f1".to_string(),
            Key::F2 => "f2".to_string(),
            Key::F3 => "f3".to_string(),
            Key::F4 => "f4".to_string(),
            Key::F5 => "f5".to_string(),
            Key::F6 => "f6".to_string(),
            Key::F7 => "f7".to_string(),
            Key::F8 => "f8".to_string(),
            Key::F9 => "f9".to_string(),
            Key::F10 => "f10".to_string(),
            Key::F11 => "f11".to_string(),
            Key::F12 => "f12".to_string(),
            Key::ArrowLeft => "left".to_string(),
            Key::ArrowRight => "right".to_string(),
            Key::ArrowUp => "up".to_string(),
            Key::ArrowDown => "down".to_string(),
            Key::Home => "home".to_string(),
            Key::End => "end".to_string(),
            Key::Insert => "insert".to_string(),
            Key::Delete => "delete".to_string(),
            Key::PageUp => "pageup".to_string(),
            Key::PageDown => "pagedown".to_string(),
            Key::Ctrl => "ctrl".to_string(),
            Key::Shift => "shift".to_string(),
            Key::Alt => "alt".to_string(),
            Key::Super => "super".to_string(),
            _ => format!("{:?}", key).to_lowercase(),
        }
    }
}