//! TCP クライアント実装
//!
//! TCP ソケットを使用してリモートサーバーと通信する機能を提供します。

use super::{NetworkClient, NetworkError, Command, Response, ConnectionInfo, ConnectionState};
use std::io::{Read, Write};
use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use serde_json;
use std::sync::{Arc, Mutex};

/// TCP クライアント
pub struct TcpClient {
    /// TCP 接続
    stream: Option<TcpStream>,
    /// 接続状態
    state: ConnectionState,
    /// 接続情報
    connection_info: Option<ConnectionInfo>,
    /// 最後に測定したレイテンシ（ミリ秒）
    latency: Option<u64>,
    /// 最後のレイテンシ測定時刻
    last_latency_check: Option<Instant>,
}

impl TcpClient {
    /// 新しい TCP クライアントを作成
    pub fn new() -> Self {
        Self {
            stream: None,
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
    
    // 内部ヘルパーメソッド
    
    /// メッセージの送信
    fn send_message(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        if let Some(stream) = &mut self.stream {
            // メッセージ長を4バイトプレフィックスとして送信
            let len = data.len() as u32;
            let len_bytes = len.to_be_bytes();
            stream.write_all(&len_bytes)?;
            stream.write_all(data)?;
            stream.flush()?;
            Ok(())
        } else {
            Err(NetworkError::ConnectionError("Not connected".to_string()))
        }
    }
    
    /// メッセージの受信
    fn receive_message(&mut self) -> Result<Vec<u8>, NetworkError> {
        if let Some(stream) = &mut self.stream {
            // メッセージ長のプレフィックスを読み取る
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes)?;
            let len = u32::from_be_bytes(len_bytes) as usize;
            
            // メッセージ本体を読み取る
            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer)?;
            
            Ok(buffer)
        } else {
            Err(NetworkError::ConnectionError("Not connected".to_string()))
        }
    }
}

impl NetworkClient for TcpClient {
    fn connect(&mut self, info: &ConnectionInfo) -> Result<(), NetworkError> {
        self.state = ConnectionState::Connecting;
        
        let addr = SocketAddr::new(
            info.host.parse().map_err(|e| NetworkError::ConnectionError(format!("Invalid host: {}", e)))?,
            info.port
        );
        
        let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(info.timeout_ms))
            .map_err(|e| NetworkError::ConnectionError(format!("Connection failed: {}", e)))?;
        
        // ノンブロッキングモードを無効化
        stream.set_nonblocking(false)?;
        
        // TCP_NODELAY フラグを設定（Nagle アルゴリズムを無効化）
        stream.set_nodelay(true)?;
        
        // バッファサイズを設定
        stream.set_recv_buffer_size(1024 * 1024 * 10)?; // 10MB
        stream.set_send_buffer_size(1024 * 1024)?;      // 1MB
        
        self.stream = Some(stream);
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
        }
        
        self.stream = None;
        self.state = ConnectionState::Disconnected;
        self.latency = None;
        
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.stream.is_some() && self.state == ConnectionState::Connected
    }
    
    fn send(&mut self, command: Command) -> Result<(), NetworkError> {
        let data = serde_json::to_vec(&command)
            .map_err(|e| NetworkError::ProtocolError(format!("Serialization error: {}", e)))?;
        
        self.send_message(&data)
    }
    
    fn receive(&mut self) -> Result<Response, NetworkError> {
        let data = self.receive_message()?;
        
        serde_json::from_slice(&data)
            .map_err(|e| NetworkError::ProtocolError(format!("Deserialization error: {}", e)))
    }
    
    fn state(&self) -> ConnectionState {
        self.state
    }
    
    fn latency(&self) -> Option<u64> {
        self.latency
    }
}

impl Default for TcpClient {
    fn default() -> Self {
        Self::new()
    }
}