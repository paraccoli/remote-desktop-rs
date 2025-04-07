//! WebSocket クライアント実装
//!
//! WebSocket を使用してリモートサーバーと通信する機能を提供します。

use super::{NetworkClient, NetworkError, Command, Response, ConnectionInfo, ConnectionState};
use std::time::{Duration, Instant};
use serde_json;
use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use url::Url;
use std::net::TcpStream;

/// WebSocket クライアント
pub struct WebSocketClient {
    /// WebSocket 接続
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    /// 接続状態
    state: ConnectionState,
    /// 接続情報
    connection_info: Option<ConnectionInfo>,
    /// 最後に測定したレイテンシ（ミリ秒）
    latency: Option<u64>,
    /// 最後のレイテンシ測定時刻
    last_latency_check: Option<Instant>,
}

impl WebSocketClient {
    /// 新しい WebSocket クライアントを作成
    pub fn new() -> Self {
        Self {
            socket: None,
            state: ConnectionState::Disconnected,
            connection_info: None,
            latency: None,
            last_latency_check: None,
        }
    }
    
    /// 指定されたホストとポートで接続
    pub fn connect_to(host: &str, port: u16) -> Result<Self, NetworkError> {
        let mut client = Self::new();
        let info = ConnectionInfo {
            host: host.to_string(),
            port,
            ..Default::default()
        };
        client.connect(&info)?;
        Ok(client)
    }
    
    /// レイテンシをチェック
    pub fn check_latency(&mut self) -> Result<u64, NetworkError> {
        if !self.is_connected() {
            return Err(NetworkError::ConnectionError("Not connected".to_string()));
        }
        
        // 最後のチェックから1秒以上経過している場合のみ更新
        let now = Instant::now();
        if let Some(last_check) = self.last_latency_check {
            if now.duration_since(last_check) < Duration::from_secs(1) {
                return Ok(self.latency.unwrap_or(0));
            }
        }
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let start = Instant::now();
        self.send(Command::Ping { timestamp })?;
        
        if let Response::Pong { original_timestamp, .. } = self.receive()? {
            if original_timestamp == timestamp {
                let latency = start.elapsed().as_millis() as u64;
                self.latency = Some(latency);
                self.last_latency_check = Some(now);
                return Ok(latency);
            }
        }
        
        Err(NetworkError::ProtocolError("Invalid pong response".to_string()))
    }
}

impl NetworkClient for WebSocketClient {
    fn connect(&mut self, info: &ConnectionInfo) -> Result<(), NetworkError> {
        self.state = ConnectionState::Connecting;
        
        // WebSocket URL を構築
        let scheme = if info.use_tls { "wss" } else { "ws" };
        let url_str = format!("{}://{}:{}/", scheme, info.host, info.port);
        let url = Url::parse(&url_str)
            .map_err(|e| NetworkError::ConnectionError(format!("Invalid URL: {}", e)))?;
        
        // WebSocket に接続
        let (socket, _) = connect(url)
            .map_err(|e| NetworkError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;
        
        self.socket = Some(socket);
        self.connection_info = Some(info.clone());
        self.state = ConnectionState::Connected;
        
        // 認証が必要な場合
        if info.username.is_some() && info.password.is_some() {
            self.state = ConnectionState::Authenticating;
            
            self.send(Command::Authenticate {
                username: info.username.clone().unwrap(),
                password: info.password.clone().unwrap(),
            })?;
            
            match self.receive()? {
                Response::AuthResult { success, message } => {
                    if success {
                        self.state = ConnectionState::Connected;
                    } else {
                        self.state = ConnectionState::Error;
                        return Err(NetworkError::AuthenticationError(message));
                    }
                },
                _ => {
                    self.state = ConnectionState::Error;
                    return Err(NetworkError::ProtocolError("Unexpected authentication response".to_string()));
                }
            }
        }
        
        // 初期レイテンシチェック
        let _ = self.check_latency();
        
        Ok(())
    }
    
    fn disconnect(&mut self) -> Result<(), NetworkError> {
        if self.is_connected() {
            // 切断メッセージを送信
            let _ = self.send(Command::Disconnect);
            
            // WebSocket を閉じる
            if let Some(socket) = &mut self.socket {
                let _ = socket.close(None);
            }
        }
        
        self.socket = None;
        self.state = ConnectionState::Disconnected;
        self.latency = None;
        
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.socket.is_some() && self.state == ConnectionState::Connected
    }
    
    fn send(&mut self, command: Command) -> Result<(), NetworkError> {
        if let Some(socket) = &mut self.socket {
            let data = serde_json::to_string(&command)
                .map_err(|e| NetworkError::ProtocolError(format!("Serialization error: {}", e)))?;
            
            socket.write_message(Message::Text(data))
                .map_err(|e| NetworkError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            
            Ok(())
        } else {
            Err(NetworkError::ConnectionError("Not connected".to_string()))
        }
    }
    
    fn receive(&mut self) -> Result<Response, NetworkError> {
        if let Some(socket) = &mut self.socket {
            let message = socket.read_message()
                .map_err(|e| NetworkError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            
            match message {
                Message::Text(text) => {
                    serde_json::from_str(&text)
                        .map_err(|e| NetworkError::ProtocolError(format!("Deserialization error: {}", e)))
                },
                Message::Binary(data) => {
                    serde_json::from_slice(&data)
                        .map_err(|e| NetworkError::ProtocolError(format!("Deserialization error: {}", e)))
                },
                Message::Close(_) => {
                    self.state = ConnectionState::Disconnected;
                    self.socket = None;
                    Err(NetworkError::ConnectionError("Connection closed by server".to_string()))
                },
                _ => Err(NetworkError::ProtocolError("Received unexpected message type".to_string())),
            }
        } else {
            Err(NetworkError::ConnectionError("Not connected".to_string()))
        }
    }
    
    fn state(&self) -> ConnectionState {
        self.state
    }
    
    fn latency(&self) -> Option<u64> {
        self.latency
    }
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}