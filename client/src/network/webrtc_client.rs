//! WebRTC クライアント実装
//!
//! WebRTC を使用してリモートサーバーと通信する機能を提供します。
//! これにより、Web版クライアントへの対応や、よりリアルタイム性の高い通信が可能になります。

use super::{NetworkClient, NetworkError, Command, Response, ConnectionInfo, ConnectionState};
use std::time::{Duration, Instant};
use serde_json;
use webrtc::api::API;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::data_channel::RTCDataChannel;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use webrtc::ice_transport::ice_server::RTCIceServer;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// WebRTC クライアント
pub struct WebRtcClient {
    /// WebRTC ピア接続
    peer_connection: Option<Arc<RTCPeerConnection>>,
    /// データチャネル
    data_channel: Option<Arc<RTCDataChannel>>,
    /// Tokio ランタイム
    runtime: Runtime,
    /// メッセージ受信キュー
    message_rx: Option<mpsc::Receiver<Vec<u8>>>,
    /// 接続状態
    state: ConnectionState,
    /// 接続情報
    connection_info: Option<ConnectionInfo>,
    /// 最後に測定したレイテンシ（ミリ秒）
    latency: Option<u64>,
    /// 最後のレイテンシ測定時刻
    last_latency_check: Option<Instant>,
    /// 接続済みフラグ
    connected: Arc<AtomicBool>,
}

impl WebRtcClient {
    /// 新しい WebRTC クライアントを作成
    pub fn new() -> Result<Self, NetworkError> {
        // Tokio ランタイムを初期化
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| NetworkError::Other(format!("Failed to create Tokio runtime: {}", e)))?;
        
        Ok(Self {
            peer_connection: None,
            data_channel: None,
            runtime,
            message_rx: None,
            state: ConnectionState::Disconnected,
            connection_info: None,
            latency: None,
            last_latency_check: None,
            connected: Arc::new(AtomicBool::new(false)),
        })
    }
    
    /// 指定されたシグナリングサーバーを使用して接続
    pub fn connect_to(signaling_url: &str) -> Result<Self, NetworkError> {
        let mut client = Self::new()?;
        let info = ConnectionInfo {
            host: signaling_url.to_string(),
            port: 0, // WebRTCではシグナリングサーバーのURLが重要
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
    
    /// シグナリングチャネルを通じてSDPオファーを送信
    fn send_offer(&self, offer: &str) -> Result<(), NetworkError> {
        // 実際の実装ではシグナリングサーバーにHTTPリクエストを送るなどの処理が必要
        // この例では簡略化のため省略
        Ok(())
    }
    
    /// シグナリングチャネルからSDPアンサーを受信
    fn receive_answer(&self) -> Result<String, NetworkError> {
        // 実際の実装ではシグナリングサーバーからのレスポンスを待つなどの処理が必要
        // この例では簡略化のため仮のアンサーを返す
        Ok("dummy_answer".to_string())
    }
}

impl NetworkClient for WebRtcClient {
    fn connect(&mut self, info: &ConnectionInfo) -> Result<(), NetworkError> {
        self.state = ConnectionState::Connecting;
        
        // WebRTC APIを初期化
        let api = API::default();
        
        // 設定を作成
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };
        
        // ピア接続を作成
        let peer_connection = self.runtime.block_on(async {
            api.new_peer_connection(config).await
        }).map_err(|e| NetworkError::ConnectionError(format!("Failed to create peer connection: {}", e)))?;
        
        let peer_connection = Arc::new(peer_connection);
        
        // データチャネルを作成
        let (message_tx, message_rx) = mpsc::channel(100);
        let connected_flag = self.connected.clone();
        
        let data_channel = self.runtime.block_on(async {
            let data_channel = peer_connection.create_data_channel(
                "remote-desktop",
                None
            ).await.map_err(|e| NetworkError::ConnectionError(format!("Failed to create data channel: {}", e)))?;
            
            // データチャネルのイベントハンドラを設定
            let dc = data_channel.clone();
            let message_tx = message_tx.clone();
            
            tokio::spawn(async move {
                let mut events = dc.on_message().await;
                while let Some(msg) = events.next().await {
                    let _ = message_tx.send(msg.data.to_vec()).await;
                }
            });
            
            let dc = data_channel.clone();
            tokio::spawn(async move {
                let mut events = dc.on_open().await;
                if events.next().await.is_some() {
                    connected_flag.store(true, Ordering::SeqCst);
                }
            });
            
            let dc = data_channel.clone();
            tokio::spawn(async move {
                let mut events = dc.on_close().await;
                if events.next().await.is_some() {
                    connected_flag.store(false, Ordering::SeqCst);
                }
            });
            
            Ok(data_channel)
        })?;
        
        // SDPオファーを作成
        let offer = self.runtime.block_on(async {
            peer_connection.create_offer(None).await
                .map_err(|e| NetworkError::ConnectionError(format!("Failed to create offer: {}", e)))?
        })?;
        
        // ローカル説明を設定
        self.runtime.block_on(async {
            peer_connection.set_local_description(offer.clone()).await
                .map_err(|e| NetworkError::ConnectionError(format!("Failed to set local description: {}", e)))?;
            
            Ok::<_, NetworkError>(())
        })?;
        
        // シグナリングサーバーにオファーを送信
        let offer_str = self.runtime.block_on(async {
            offer.sdp
        });
        self.send_offer(&offer_str)?;
        
        // シグナリングサーバーからアンサーを受信
        let answer_str = self.receive_answer()?;
        
        // リモート説明を設定
        // 実際の実装ではシグナリングサーバーから受け取ったアンサーをパースする必要がある
        // この例では簡略化のため省略
        
        self.peer_connection = Some(peer_connection);
        self.data_channel = Some(Arc::new(data_channel));
        self.message_rx = Some(message_rx);
        self.connection_info = Some(info.clone());
        
        // 接続完了まで待機
        let start_time = Instant::now();
        while !self.connected.load(Ordering::SeqCst) {
            if start_time.elapsed() > Duration::from_secs(30) {
                return Err(NetworkError::TimeoutError("Connection timeout".to_string()));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        
        self.state = ConnectionState::Connected;
        
        // 認証が必要な場合は処理を追加
        
        // 初期レイテンシチェック
        let _ = self.check_latency();
        
        Ok(())
    }
    
    fn disconnect(&mut self) -> Result<(), NetworkError> {
        if self.is_connected() {
            // 切断メッセージを送信
            let _ = self.send(Command::Disconnect);
            
            // データチャネルをクローズ
            if let Some(dc) = &self.data_channel {
                self.runtime.block_on(async {
                    let _ = dc.close().await;
                });
            }
            
            // ピア接続をクローズ
            if let Some(pc) = &self.peer_connection {
                self.runtime.block_on(async {
                    let _ = pc.close().await;
                });
            }
        }
        
        self.peer_connection = None;
        self.data_channel = None;
        self.message_rx = None;
        self.state = ConnectionState::Disconnected;
        self.latency = None;
        self.connected.store(false, Ordering::SeqCst);
        
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst) && self.state == ConnectionState::Connected
    }
    
    fn send(&mut self, command: Command) -> Result<(), NetworkError> {
        if let Some(dc) = &self.data_channel {
            let data = serde_json::to_string(&command)
                .map_err(|e| NetworkError::ProtocolError(format!("Serialization error: {}", e)))?;
            
            self.runtime.block_on(async {
                dc.send_text(&data).await
                    .map_err(|e| NetworkError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
            })?;
            
            Ok(())
        } else {
            Err(NetworkError::ConnectionError("Not connected".to_string()))
        }
    }
    
    fn receive(&mut self) -> Result<Response, NetworkError> {
        if let Some(rx) = &mut self.message_rx {
            // タイムアウト付きでメッセージを受信
            let timeout = Duration::from_secs(10);
            let data = self.runtime.block_on(async {
                tokio::time::timeout(timeout, rx.recv()).await
                    .map_err(|_| NetworkError::TimeoutError("Receive timeout".to_string()))?
                    .ok_or_else(|| NetworkError::ConnectionError("Channel closed".to_string()))
            })?;
            
            // JSONデータをデシリアライズ
            let response: Response = serde_json::from_slice(&data)
                .map_err(|e| NetworkError::ProtocolError(format!("Deserialization error: {}", e)))?;
            
            Ok(response)
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