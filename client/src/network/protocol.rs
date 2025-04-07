//! 通信プロトコル定義
//!
//! クライアントとサーバー間の通信プロトコルを定義します。

use crate::input::MouseButton;
use egui::Key;
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// サーバーに送信するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// 認証
    Authenticate {
        /// ユーザー名
        username: String,
        /// パスワード
        password: String,
    },
    
    /// スクリーンショット要求
    RequestScreenshot {
        /// 画質（1～100）
        quality: u8,
        /// 幅（オプション）
        width: Option<u32>,
        /// 高さ（オプション）
        height: Option<u32>,
        /// モニターインデックス（オプション）
        monitor: Option<usize>,
    },
    
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
    
    /// 画質設定
    SetQuality {
        /// 画質値（1～100）
        quality: u8,
    },
    
    /// アプリケーション実行
    RunApplication {
        /// コマンド
        command: String,
    },
    
    /// Ping
    Ping {
        /// タイムスタンプ（ミリ秒）
        timestamp: u64,
    },
    
    /// 切断
    Disconnect,
}

/// サーバーからのレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// 認証結果
    AuthResult {
        /// 成功したかどうか
        success: bool,
        /// メッセージ
        message: String,
    },
    
    /// スクリーンショットデータ
    ScreenshotData {
        /// 画像データ
        data: Vec<u8>,
        /// 画像フォーマット
        format: ImageFormat,
        /// 幅
        width: u32,
        /// 高さ
        height: u32,
        /// タイムスタンプ（ミリ秒）
        timestamp: u64,
    },
    
    /// コマンド実行結果
    CommandResult {
        /// 成功したかどうか
        success: bool,
        /// メッセージ
        message: String,
    },
    
    /// Pong
    Pong {
        /// 元のPingのタイムスタンプ
        original_timestamp: u64,
        /// サーバー時間（ミリ秒）
        server_time: u64,
    },
    
    /// エラー
    Error {
        /// エラーコード
        code: i32,
        /// エラーメッセージ
        message: String,
    },
}

/// 画像フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// 接続情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// ホスト名または IP アドレス
    pub host: String,
    /// ポート番号
    pub port: u16,
    /// ユーザー名（オプション）
    pub username: Option<String>,
    /// パスワード（オプション）
    pub password: Option<String>,
    /// 接続タイムアウト（ミリ秒）
    pub timeout_ms: u64,
    /// TLS を使用するかどうか
    pub use_tls: bool,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9999,
            username: None,
            password: None,
            timeout_ms: 5000,
            use_tls: false,
        }
    }
}

/// 接続状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// 切断
    Disconnected,
    /// 接続中
    Connecting,
    /// 接続済み
    Connected,
    /// 認証中
    Authenticating,
    /// エラー
    Error,
}