//! サーバー設定
//!
//! サーバーの設定情報を管理するモジュール

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// サーバー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// ネットワーク設定
    pub network: NetworkSettings,
    /// セキュリティ設定
    pub security: SecuritySettings,
    /// キャプチャ設定
    pub capture: CaptureSettings,
}

/// ネットワーク設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// バインドアドレス
    pub bind_address: String,
    /// ポート番号
    pub port: u16,
    /// TLS使用フラグ
    pub use_tls: bool,
    /// TLS証明書パス
    pub tls_cert_path: Option<PathBuf>,
    /// TLS秘密鍵パス
    pub tls_key_path: Option<PathBuf>,
    /// WebSocket有効フラグ
    pub enable_websocket: bool,
    /// WebRTC有効フラグ
    pub enable_webrtc: bool,
    /// 最大接続数
    pub max_connections: usize,
    /// クライアントタイムアウト(秒)
    pub client_timeout: u64,
    /// キープアライブ間隔(秒)
    pub keep_alive_interval: u64,
}

/// セキュリティ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// 認証要求フラグ
    pub require_auth: bool,
    /// アクセス制御設定
    pub access_control: AccessControlSettings,
}

/// アクセス制御設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlSettings {
    /// アクセス許可IPアドレスリスト
    pub allowed_ips: Vec<String>,
    /// アクセス拒否IPアドレスリスト
    pub denied_ips: Vec<String>,
}

/// キャプチャ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSettings {
    /// 画質設定(1-100)
    pub quality: u8,
    /// フレームレート
    pub frame_rate: u8,
    /// 最大画像サイズ制限
    pub max_size: Option<(u32, u32)>,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            network: NetworkSettings::default(),
            security: SecuritySettings::default(),
            capture: CaptureSettings::default(),
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 9000,
            use_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            enable_websocket: false,
            enable_webrtc: false,
            max_connections: 5,
            client_timeout: 60,
            keep_alive_interval: 10,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            require_auth: true,
            access_control: AccessControlSettings::default(),
        }
    }
}

impl Default for AccessControlSettings {
    fn default() -> Self {
        Self {
            allowed_ips: Vec::new(),
            denied_ips: Vec::new(),
        }
    }
}

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            quality: 75,
            frame_rate: 30,
            max_size: None,
        }
    }
}