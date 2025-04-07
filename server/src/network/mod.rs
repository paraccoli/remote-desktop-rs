//! ネットワークモジュール
//!
//! リモートデスクトップサーバーのネットワーク通信機能を提供します。
//! クライアントからの接続を受け付け、コマンドの処理とレスポンスの送信を行います。

pub mod tcp_server;
pub mod websocket_server;
pub mod webrtc_server;
pub mod protocol;
pub mod authentication;
pub mod session;

use remote_desktop_rs_common::protocol::{Command, Response, ClientInfo};
use crate::capture::{ScreenCapture, CapturedImage};
use crate::input::InputHandler;
use crate::error::ServerError;

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use thiserror::Error;

/// ネットワークエラー
#[derive(Error, Debug)]
pub enum NetworkError {
    /// I/Oエラー
    #[error("I/Oエラー: {0}")]
    IoError(#[from] std::io::Error),
    
    /// 通信エラー
    #[error("通信エラー: {0}")]
    CommunicationError(String),
    
    /// プロトコルエラー
    #[error("プロトコルエラー: {0}")]
    ProtocolError(String),
    
    /// タイムアウトエラー
    #[error("タイムアウト: {0}")]
    TimeoutError(String),
    
    /// 認証エラー
    #[error("認証エラー: {0}")]
    AuthenticationError(String),
    
    /// セッションエラー
    #[error("セッションエラー: {0}")]
    SessionError(String),
    
    /// スレッドエラー
    #[error("スレッドエラー: {0}")]
    ThreadError(String),
    
    /// その他のエラー
    #[error("ネットワークエラー: {0}")]
    Other(String),
}

/// サーバー設定
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// バインドするアドレス
    pub bind_address: String,
    /// ポート番号
    pub port: u16,
    /// TLSを使用するかどうか
    pub use_tls: bool,
    /// TLS証明書ファイルパス
    pub tls_cert_path: Option<String>,
    /// TLS秘密鍵ファイルパス
    pub tls_key_path: Option<String>,
    /// 認証を必要とするかどうか
    pub require_auth: bool,
    /// 最大同時接続数
    pub max_connections: usize,
    /// クライアントタイムアウト（秒）
    pub client_timeout: u64,
    /// キープアライブ間隔（秒）
    pub keep_alive_interval: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 9999,
            use_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            require_auth: false,
            max_connections: 5,
            client_timeout: 60,
            keep_alive_interval: 30,
        }
    }
}

/// ネットワークサーバー
pub trait NetworkServer {
    /// サーバーを起動
    fn start(&mut self) -> Result<(), NetworkError>;
    
    /// サーバーを停止
    fn stop(&mut self) -> Result<(), NetworkError>;
    
    /// サーバーが実行中かどうかを確認
    fn is_running(&self) -> bool;
    
    /// 接続されているクライアント数を取得
    fn connected_clients(&self) -> usize;
    
    /// アドレスを取得
    fn get_address(&self) -> SocketAddr;
}

/// サーバーファクトリー
pub struct ServerFactory;

impl ServerFactory {
    /// サーバーを作成
    pub fn create_server(
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
    ) -> Result<Box<dyn NetworkServer + Send>, NetworkError> {
        let server: Box<dyn NetworkServer + Send> = match config.use_tls {
            true => {
                if config.tls_cert_path.is_none() || config.tls_key_path.is_none() {
                    return Err(NetworkError::Other("TLSを使用するには証明書と秘密鍵が必要です".to_string()));
                }
                
                Box::new(tcp_server::TlsTcpServer::new(
                    config.clone(),
                    screen_capture,
                    input_handler,
                )?)
            },
            false => {
                Box::new(tcp_server::TcpServer::new(
                    config.clone(),
                    screen_capture,
                    input_handler,
                )?)
            }
        };
        
        Ok(server)
    }
    
    /// WebSocketサーバーを作成
    pub fn create_websocket_server(
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
    ) -> Result<Box<dyn NetworkServer + Send>, NetworkError> {
        let server = websocket_server::WebSocketServer::new(
            config.clone(),
            screen_capture,
            input_handler,
        )?;
        
        Ok(Box::new(server))
    }
    
    /// WebRTCサーバーを作成
    #[cfg(feature = "webrtc-support")]
    pub fn create_webrtc_server(
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
    ) -> Result<Box<dyn NetworkServer + Send>, NetworkError> {
        let server = webrtc_server::WebRtcServer::new(
            config.clone(),
            screen_capture,
            input_handler,
        )?;
        
        Ok(Box::new(server))
    }
}

/// セッション情報
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// セッションID
    pub id: String,
    /// クライアント情報
    pub client_info: Option<ClientInfo>,
    /// IPアドレス
    pub ip_address: String,
    /// 認証済みかどうか
    pub authenticated: bool,
    /// 接続日時
    pub connection_time: std::time::SystemTime,
    /// 最後のアクティビティ
    pub last_activity: std::time::SystemTime,
    /// 送信バイト数
    pub bytes_sent: u64,
    /// 受信バイト数
    pub bytes_received: u64,
    /// 画質設定
    pub quality: u8,
    /// 最後のレイテンシー（ms）
    pub last_latency: Option<u64>,
}

impl SessionInfo {
    /// 新しいセッション情報を作成
    pub fn new(id: String, ip_address: String) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id,
            client_info: None,
            ip_address,
            authenticated: false,
            connection_time: now,
            last_activity: now,
            bytes_sent: 0,
            bytes_received: 0,
            quality: 70,
            last_latency: None,
        }
    }
    
    /// アクティビティを更新
    pub fn update_activity(&mut self) {
        self.last_activity = std::time::SystemTime::now();
    }
    
    /// アイドル時間を取得（秒）
    pub fn idle_time(&self) -> u64 {
        self.last_activity
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
    
    /// 接続時間を取得（秒）
    pub fn connection_duration(&self) -> u64 {
        self.connection_time
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}