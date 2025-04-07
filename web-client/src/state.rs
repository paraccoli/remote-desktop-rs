//! アプリケーション状態管理
//!
//! Webクライアントのアプリケーション状態を管理するモジュールです。

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wasm_bindgen::prelude::*;
use web_sys::{WebSocket, RtcPeerConnection, RtcDataChannel};

/// アプリケーション設定
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// 最大FPS
    pub max_fps: u8,
    /// ビデオ品質 (1-100)
    pub video_quality: u8,
    /// 暗号化を有効化
    pub enable_encryption: bool,
    /// WebRTCを優先
    pub prefer_webrtc: bool,
    /// キャプチャ形式
    pub capture_format: String,
    /// コントロールオプション
    pub control_options: ControlOptions,
    /// 表示オプション
    pub display_options: DisplayOptions,
    /// 接続履歴を保存
    pub save_connection_history: bool,
    /// 最近の接続数を保存
    pub max_connection_history: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_fps: 30,
            video_quality: 80,
            enable_encryption: true,
            prefer_webrtc: true,
            capture_format: "webp".to_string(),
            control_options: ControlOptions::default(),
            display_options: DisplayOptions::default(),
            save_connection_history: true,
            max_connection_history: 10,
        }
    }
}

/// 接続情報
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ConnectionInfo {
    /// ホスト名またはIPアドレス
    pub host: String,
    /// ポート番号
    pub port: u16,
    /// プロトコル (websocket, webrtc)
    pub protocol: String,
    /// TLS使用フラグ
    pub use_tls: bool,
    /// WebRTC優先フラグ
    pub prefer_webrtc: bool,
    /// ユーザー名 (認証時)
    pub username: Option<String>,
    /// パスワード (認証時)
    pub password: Option<String>,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9090,
            protocol: "websocket".to_string(),
            use_tls: false,
            prefer_webrtc: true,
            username: None,
            password: None,
        }
    }
}

/// コントロールオプション
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ControlOptions {
    /// クリップボード同期を有効化
    pub enable_clipboard_sync: bool,
    /// ファイル転送を有効化
    pub enable_file_transfer: bool,
    /// キー入力時の自動フォーカス
    pub auto_focus_on_keyboard: bool,
    /// マウスホイールの速度係数
    pub mouse_wheel_speed: f32,
    /// 接続時にCtrl+Alt+Delを送信
    pub send_cad_on_connect: bool,
}

impl Default for ControlOptions {
    fn default() -> Self {
        Self {
            enable_clipboard_sync: true,
            enable_file_transfer: true,
            auto_focus_on_keyboard: true,
            mouse_wheel_speed: 1.0,
            send_cad_on_connect: false,
        }
    }
}

/// 表示オプション
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayOptions {
    /// 接続時にフルスクリーン表示
    pub fullscreen_on_connect: bool,
    /// アスペクト比を維持
    pub maintain_aspect_ratio: bool,
    /// スケーリング方法
    pub scaling_method: String,
    /// カーソルを表示
    pub show_remote_cursor: bool,
    /// ダークモード
    pub dark_mode: bool,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            fullscreen_on_connect: false,
            maintain_aspect_ratio: true,
            scaling_method: "bilinear".to_string(),
            show_remote_cursor: true,
            dark_mode: false,
        }
    }
}

/// アプリケーション状態
#[derive(Clone, Debug)]
pub enum AppState {
    /// 初期状態
    Initial,
    /// 接続中
    Connecting,
    /// 接続済み
    Connected,
    /// 認証中
    Authenticating,
    /// エラー
    Error(String),
    /// 切断
    Disconnected,
}

impl Default for AppState {
    fn default() -> Self {
        Self::Initial
    }
}