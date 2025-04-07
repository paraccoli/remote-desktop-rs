//! 設定パネル
//!
//! サーバー設定のUIを実装します。

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
    /// ログ設定
    pub logging: LoggingSettings,
}

/// ネットワーク設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// バインドアドレス
    pub bind_address: String,
    /// ポート番号
    pub port: u16,
    /// 最大接続数
    pub max_connections: usize,
    /// クライアントタイムアウト（秒）
    pub client_timeout: u64,
    /// キープアライブ間隔（秒）
    pub keep_alive_interval: u64,
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
}

/// セキュリティ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// 認証必須フラグ
    pub require_auth: bool,
    /// 接続IPアドレス制限
    pub allowed_ips: Vec<String>,
    /// クリップボード共有許可
    pub allow_clipboard: bool,
    /// ファイル転送許可
    pub allow_file_transfer: bool,
    /// アプリケーション実行許可
    pub allow_application_launch: bool,
    /// ユーザー/パスワードのリスト
    pub users: Vec<UserCredential>,
}

/// キャプチャ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSettings {
    /// デフォルト画質（1-100）
    pub default_quality: u8,
    /// デフォルト画像形式
    pub default_format: String,
    /// キャプチャFPS制限
    pub max_fps: u8,
    /// モニター選択
    pub monitor_index: Option<usize>,
    /// キャプチャ領域
    pub capture_region: Option<CaptureRegion>,
}

/// ログ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// ログレベル
    pub log_level: String,
    /// ログファイルパス
    pub log_file: Option<PathBuf>,
    /// 最大ログサイズ（バイト）
    pub max_log_size: u64,
    /// ログローテーション数
    pub max_log_files: u8,
    /// ログに接続情報を含めるフラグ
    pub log_connections: bool,
    /// ログにコマンド情報を含めるフラグ
    pub log_commands: bool,
}

/// ユーザー認証情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredential {
    /// ユーザー名
    pub username: String,
    /// パスワードハッシュ
    pub password_hash: String,
    /// 権限レベル（0=ビューアー、1=制御者、2=管理者）
    pub permission_level: u8,
}

/// キャプチャ領域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRegion {
    /// X座標
    pub x: i32,
    /// Y座標
    pub y: i32,
    /// 幅
    pub width: u32,
    /// 高さ
    pub height: u32,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            network: NetworkSettings::default(),
            security: SecuritySettings::default(),
            capture: CaptureSettings::default(),
            logging: LoggingSettings::default(),
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 9090,
            max_connections: 5,
            client_timeout: 300,
            keep_alive_interval: 30,
            use_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            enable_websocket: true,
            enable_webrtc: false,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            require_auth: true,
            allowed_ips: Vec::new(),
            allow_clipboard: true,
            allow_file_transfer: false,
            allow_application_launch: false,
            users: vec![
                UserCredential {
                    username: "admin".to_string(),
                    password_hash: "5f4dcc3b5aa765d61d8327deb882cf99".to_string(), // "password"のMD5ハッシュ
                    permission_level: 2,
                }
            ],
        }
    }
}

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            default_quality: 75,
            default_format: "jpeg".to_string(),
            max_fps: 30,
            monitor_index: None,
            capture_region: None,
        }
    }
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_file: Some(PathBuf::from("logs/server.log")),
            max_log_size: 10 * 1024 * 1024, // 10MB
            max_log_files: 5,
            log_connections: true,
            log_commands: false,
        }
    }
}

impl ServerSettings {
    /// 設定をファイルから読み込む
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(settings) => Ok(settings),
                    Err(e) => Err(format!("設定ファイルのパースに失敗しました: {}", e))
                }
            },
            Err(e) => Err(format!("設定ファイルの読み込みに失敗しました: {}", e))
        }
    }
    
    /// 設定をファイルに保存
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        // 親ディレクトリが存在しない場合は作成
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Err(format!("設定ディレクトリの作成に失敗しました: {}", e));
                }
            }
        }
        
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                match std::fs::write(path, json) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("設定ファイルの書き込みに失敗しました: {}", e))
                }
            },
            Err(e) => Err(format!("設定のシリアライズに失敗しました: {}", e))
        }
    }
    
    /// ネットワーク設定を取得
    pub fn get_network(&self) -> &NetworkSettings {
        &self.network
    }
    
    /// セキュリティ設定を取得
    pub fn get_security(&self) -> &SecuritySettings {
        &self.security
    }
    
    /// キャプチャ設定を取得
    pub fn get_capture(&self) -> &CaptureSettings {
        &self.capture
    }
    
    /// ログ設定を取得
    pub fn get_logging(&self) -> &LoggingSettings {
        &self.logging
    }
}