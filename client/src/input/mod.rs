//! 入力モジュール
//! 
//! このモジュールはリモートデスクトップクライアントの入力処理を担当します。
//! マウスやキーボードからの入力イベントを処理し、適切なプロトコルコマンドに変換します。

mod events;
mod mapping;

pub use events::{InputEventHandler, InputEvent};
pub use mapping::{KeyMapping, MouseMapping};

/// マウスボタン
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// 左ボタン
    Left,
    /// 右ボタン
    Right,
    /// 中ボタン
    Middle,
    /// 拡張ボタン1
    X1,
    /// 拡張ボタン2
    X2,
}

impl MouseButton {
    /// プロトコル用の文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            MouseButton::Left => "left",
            MouseButton::Right => "right",
            MouseButton::Middle => "middle",
            MouseButton::X1 => "x1",
            MouseButton::X2 => "x2",
        }
    }
}

/// キーボードの修飾キー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    /// Ctrlキー
    Ctrl,
    /// Altキー
    Alt,
    /// Shiftキー
    Shift,
    /// Winキー/Commandキー
    Super,
}

impl Modifier {
    /// プロトコル用の文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Modifier::Ctrl => "ctrl",
            Modifier::Alt => "alt",
            Modifier::Shift => "shift",
            Modifier::Super => "super",
        }
    }
}

/// 入力モードの設定
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// 通常モード - すべての入力をリモートに送信
    Normal,
    /// 表示専用モード - 入力をリモートに送信しない
    ViewOnly,
    /// 部分的モード - 特定の入力のみリモートに送信
    Partial,
}