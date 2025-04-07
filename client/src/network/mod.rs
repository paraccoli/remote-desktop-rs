//! ネットワークモジュール
//!
//! リモートサーバーとの通信を担当する機能を提供します。
//! TCP、WebSocket、WebRTCなど様々な通信プロトコルをサポートします。

mod tcp_client;
mod websocket_client;
mod webrtc_client;
mod protocol;

pub use tcp_client::TcpClient;
pub use websocket_client::WebSocketClient;
pub use webrtc_client::WebRtcClient;
pub use protocol::{Command, Response, ConnectionInfo, ConnectionState};

use thiserror::Error;
use std::io;

/// ネットワークエラー
#[derive(Error, Debug)]
pub enum NetworkError {
    /// 接続エラー
    #[error("接続エラー: {0}")]
    ConnectionError(String),
    
    /// IO エラー
    #[error("IO エラー: {0}")]
    IoError(#[from] io::Error),
    
    /// プロトコルエラー
    #[error("プロトコルエラー: {0}")]
    ProtocolError(String),
    
    /// タイムアウトエラー
    #[error("タイムアウト: {0}")]
    TimeoutError(String),
    
    /// 認証エラー
    #[error("認証エラー: {0}")]
    AuthenticationError(String),
    
    /// その他のエラー
    #[error("ネットワークエラー: {0}")]
    Other(String),
}

/// ネットワークインターフェース
///
/// すべての通信クライアントが実装する必要があるトレイト
pub trait NetworkClient {
    /// サーバーに接続
    fn connect(&mut self, info: &ConnectionInfo) -> Result<(), NetworkError>;
    
    /// サーバーから切断
    fn disconnect(&mut self) -> Result<(), NetworkError>;
    
    /// サーバーに接続されているかどうかを確認
    fn is_connected(&self) -> bool;
    
    /// コマンドを送信
    fn send(&mut self, command: Command) -> Result<(), NetworkError>;
    
    /// レスポンスを受信
    fn receive(&mut self) -> Result<Response, NetworkError>;
    
    /// 接続状態を取得
    fn state(&self) -> ConnectionState;
    
    /// レイテンシを取得 (ミリ秒)
    fn latency(&self) -> Option<u64>;
}