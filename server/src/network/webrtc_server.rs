//! WebRTC サーバー実装
//!
//! WebRTC を使用してクライアントと通信するサーバーを実装します。
//! これにより、Webブラウザからの接続や、特定のNAT環境でも接続できる可能性が高まります。

use super::{NetworkServer, NetworkError, ServerConfig, SessionInfo};
use super::session::{ClientSession, ClientConnection};
use super::authentication::Authenticator;
use remote_desktop_rs_common::protocol::{Command, Response};
use crate::capture::ScreenCapture;
use crate::input::InputHandler;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use serde_json;
use log::{debug, info, warn, error};

// WebRTC関連ライブラリ
#[cfg(feature = "webrtc-support")]
use webrtc::api::API;
#[cfg(feature = "webrtc-support")]
use webrtc::peer_connection::configuration::RTCConfiguration;
#[cfg(feature = "webrtc-support")]
use webrtc::peer_connection::RTCPeerConnection;
#[cfg(feature = "webrtc-support")]
use webrtc::data_channel::RTCDataChannel;
#[cfg(feature = "webrtc-support")]
use webrtc::ice_transport::ice_server::RTCIceServer;
#[cfg(feature = "webrtc-support")]
use tokio::runtime::Runtime;
#[cfg(feature = "webrtc-support")]
use tokio::sync::mpsc;

/// WebRTC サーバー
pub struct WebRtcServer {
    /// サーバー設定
    config: ServerConfig,
    /// スクリーンキャプチャー
    screen_capture: Arc<Mutex<ScreenCapture>>,
    /// 入力ハンドラー
    input_handler: Arc<Mutex<InputHandler>>,
    /// 認証ハンドラ
    authenticator: Authenticator,
    /// シグナリングサーバースレッド
    signaling_thread: Option<thread::JoinHandle<()>>,
    /// WebRTCワーカースレッド
    worker_thread: Option<thread::JoinHandle<()>>,
    /// スレッド管理用チャネル
    thread_control: Option<mpsc::Sender<()>>,
    /// クライアントセッションリスト
    client_sessions: Arc<Mutex<Vec<Arc<Mutex<ClientSession>>>>>,
    /// 起動中フラグ
    running: Arc<Mutex<bool>>,
    /// Tokio ランタイム
    #[cfg(feature = "webrtc-support")]
    runtime: Option<Arc<Runtime>>,
    /// サーバーアドレス
    server_addr: Option<SocketAddr>,
}

#[cfg(feature = "webrtc-support")]
impl WebRtcServer {
    /// 新しいWebRTCサーバーを作成
    pub fn new(
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
    ) -> Result<Self, NetworkError> {
        // Tokio ランタイムを初期化
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| NetworkError::Other(format!("Tokioランタイムの作成に失敗: {}", e)))?;

        Ok(Self {
            config,
            screen_capture,
            input_handler,
            authenticator: Authenticator::new(),
            signaling_thread: None,
            worker_thread: None,
            thread_control: None,
            client_sessions: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            runtime: Some(Arc::new(runtime)),
            server_addr: None,
        })
    }

    /// シグナリングサーバーを起動
    fn start_signaling_server(&mut self) -> Result<(), NetworkError> {
        // バインドアドレス
        let bind_addr = format!("{}:{}", self.config.bind_address, self.config.port);
        
        // TCPリスナーを作成
        let listener = TcpListener::bind(&bind_addr)
            .map_err(|e| NetworkError::IoError(e))?;
        
        // 非ブロッキングモードに設定
        listener.set_nonblocking(true)
            .map_err(|e| NetworkError::IoError(e))?;
            
        self.server_addr = Some(listener.local_addr()?);
        
        let (tx, rx) = mpsc::channel();
        self.thread_control = Some(tx);
        
        // 共有データの複製
        let screen_capture = self.screen_capture.clone();
        let input_handler = self.input_handler.clone();
        let authenticator = Arc::new(self.authenticator.clone());
        let config = self.config.clone();
        let client_sessions = self.client_sessions.clone();
        let running = self.running.clone();
        let runtime = self.runtime.as_ref().unwrap().clone();
        
        // シグナリングサーバースレッドを起動
        let signaling_thread = thread::spawn(move || {
            // 実行中フラグをセット
            *running.lock().unwrap() = true;
            
            info!("WebRTCシグナリングサーバー起動: {}", bind_addr);
            
            // 接続を受け付けるループ
            let mut last_cleanup = Instant::now();
            
            loop {
                // サーバー終了要求をチェック
                if let Ok(_) = rx.try_recv() {
                    break;
                }
                
                // 接続を受け付け
                match listener.accept() {
                    Ok((stream, addr)) => {
                        // 最大接続数のチェック
                        let session_count = client_sessions.lock().unwrap().len();
                        if session_count >= config.max_connections {
                            warn!("最大接続数到達: {}/{}", session_count, config.max_connections);
                            continue;
                        }
                        
                        // シグナリング処理を開始
                        Self::handle_signaling(
                            stream,
                            addr,
                            config.clone(),
                            screen_capture.clone(),
                            input_handler.clone(),
                            authenticator.clone(),
                            client_sessions.clone(),
                            runtime.clone(),
                        );
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // 接続要求がない場合は短いスリープ
                        thread::sleep(Duration::from_millis(100));
                    },
                    Err(e) => {
                        error!("接続受付エラー: {}", e);
                        // 短いスリープを入れて連続エラーを防止
                        thread::sleep(Duration::from_millis(1000));
                    }
                }
                
                // 定期的にアイドルセッションのクリーンアップを行う（30秒ごと）
                if last_cleanup.elapsed() > Duration::from_secs(30) {
                    // タイムアウトしたセッションを削除
                    let mut sessions = client_sessions.lock().unwrap();
                    sessions.retain(|session| {
                        let s = session.lock().unwrap();
                        s.idle_time() < config.client_timeout
                    });
                    
                    last_cleanup = Instant::now();
                }
            }
            
            info!("WebRTCシグナリングサーバー停止");
            
            // 実行中フラグをクリア
            *running.lock().unwrap() = false;
        });
        
        self.signaling_thread = Some(signaling_thread);
        
        Ok(())
    }
    
    /// シグナリング処理を行う
    fn handle_signaling(
        stream: TcpStream,
        client_addr: SocketAddr,
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
        authenticator: Arc<Authenticator>,
        client_sessions: Arc<Mutex<Vec<Arc<Mutex<ClientSession>>>>>,
        runtime: Arc<Runtime>,
    ) {
        // この実装は簡略化しています
        // 実際のシグナリングプロトコルはより複雑になります
        
        thread::spawn(move || {
            info!("WebRTCシグナリング接続: {}", client_addr);
            
            // クライアントからのオファーを受信
            let mut buffer = [0; 8192];
            let mut stream = stream;
            
            // タイムアウトを設定
            let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
            let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
            
            match stream.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    // オファーデータを解析
                    if let Ok(offer_str) = std::str::from_utf8(&buffer[0..bytes_read]) {
                        // WebRTC接続処理
                        if let Err(e) = Self::setup_webrtc_connection(
                            offer_str,
                            client_addr,
                            config,
                            screen_capture,
                            input_handler,
                            authenticator,
                            client_sessions,
                            runtime,
                            &mut stream,
                        ) {
                            error!("WebRTC接続のセットアップに失敗: {}", e);
                        }
                    }
                },
                _ => {
                    error!("オファーの受信に失敗");
                }
            }
            
            info!("WebRTCシグナリング処理終了: {}", client_addr);
        });
    }
    
    /// WebRTC接続をセットアップする
    fn setup_webrtc_connection(
        offer_sdp: &str,
        client_addr: SocketAddr,
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
        authenticator: Arc<Authenticator>,
        client_sessions: Arc<Mutex<Vec<Arc<Mutex<ClientSession>>>>>,
        runtime: Arc<Runtime>,
        signaling_stream: &mut TcpStream,
    ) -> Result<(), NetworkError> {
        // WebRTC APIを初期化
        let api = API::default();
        
        // ICE設定
        let rtc_config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };
        
        // ピア接続を作成
        let peer_connection = runtime.block_on(async {
            api.new_peer_connection(rtc_config).await
        }).map_err(|e| NetworkError::Other(format!("ピア接続の作成に失敗: {}", e)))?;
        
        let peer_connection = Arc::new(peer_connection);
        
        // WebRTCクライアント接続用のチャネル
        let (message_tx, message_rx) = tokio::sync::mpsc::channel(100);
        
        // データチャネルイベントハンドラを設定
        let pc = peer_connection.clone();
        runtime.spawn(async move {
            // イベントの登録
            let mut data_channel_opened = pc.on_data_channel().await;
            
            while let Some(data_channel) = data_channel_opened.next().await {
                let message_tx = message_tx.clone();
                
                // メッセージ受信ハンドラ
                let dc = data_channel.clone();
                tokio::spawn(async move {
                    let mut message_events = dc.on_message().await;
                    while let Some(msg) = message_events.next().await {
                        let _ = message_tx.send(msg.data.to_vec()).await;
                    }
                });
            }
        });
        
        // リモート記述を設定
        runtime.block_on(async {
            let desc = webrtc::peer_connection::sdp::session_description::RTCSessionDescription {
                sdp_type: webrtc::peer_connection::sdp::session_description::RTCSdpType::Offer,
                sdp: offer_sdp.to_string(),
            };
            
            peer_connection.set_remote_description(desc).await
        }).map_err(|e| NetworkError::Other(format!("リモート記述の設定に失敗: {}", e)))?;
        
        // アンサーを作成
        let answer = runtime.block_on(async {
            peer_connection.create_answer(None).await
        }).map_err(|e| NetworkError::Other(format!("アンサーの作成に失敗: {}", e)))?;
        
        // ローカル記述を設定
        runtime.block_on(async {
            peer_connection.set_local_description(answer.clone()).await
        }).map_err(|e| NetworkError::Other(format!("ローカル記述の設定に失敗: {}", e)))?;
        
        // アンサーをシグナリングチャネルに送信
        let answer_str = runtime.block_on(async { answer.sdp });
        signaling_stream.write_all(answer_str.as_bytes())?;
        signaling_stream.flush()?;
        
        // クライアント接続を作成
        let client_connection = WebRtcClientConnection {
            runtime: runtime.clone(),
            peer_connection: peer_connection.clone(),
            message_rx: Some(message_rx),
            message_buffer: Vec::new(),
        };
        
        // セッション情報を作成
        let session_id = uuid::Uuid::new_v4().to_string();
        let session_info = SessionInfo::new(session_id.clone(), client_addr.to_string());
        
        // WebRTC用のクライアントセッションを作成
        let session = ClientSession::new(
            session_info,
            Box::new(client_connection),
            screen_capture,
            input_handler,
            authenticator,
            config,
        );
        
        let session = Arc::new(Mutex::new(session));
        
        // セッションリストに追加
        {
            let mut sessions = client_sessions.lock().unwrap();
            sessions.push(session.clone());
        }
        
        // WebRTC接続処理スレッドを起動
        let session_clone = session.clone();
        runtime.spawn(async move {
            // セッション処理ループ
            loop {
                // 短いスリープ
                tokio::time::sleep(Duration::from_millis(10)).await;
                
                // セッションを処理
                let mut session = session_clone.lock().unwrap();
                
                // セッションがアクティブでなければ終了
                if !session.is_active() {
                    break;
                }
                
                // コマンドの受信と処理
                if let Err(e) = session.receive_and_process() {
                    if let NetworkError::IoError(ref io_err) = e {
                        if io_err.kind() == std::io::ErrorKind::WouldBlock || 
                           io_err.kind() == std::io::ErrorKind::TimedOut {
                            // タイムアウト - 無視してループ継続
                            continue;
                        }
                    }
                    
                    // その他のエラー - セッション終了
                    error!("WebRTC接続エラー: {}", e);
                    break;
                }
            }
            
            info!("WebRTC接続終了: {}", client_addr);
        });
        
        Ok(())
    }
}

/// トレイト実装：NetworkServer
#[cfg(feature = "webrtc-support")]
impl NetworkServer for WebRtcServer {
    fn start(&mut self) -> Result<(), NetworkError> {
        // 既に起動していれば何もしない
        if self.is_running() {
            return Ok(());
        }
        
        // シグナリングサーバーを起動
        self.start_signaling_server()?;
        
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), NetworkError> {
        // 起動していなければ何もしない
        if !self.is_running() {
            return Ok(());
        }
        
        // スレッド終了シグナルを送信
        if let Some(tx) = &self.thread_control {
            let _ = tx.send(());
        }
        
        // シグナリングスレッドの終了を待機
        if let Some(handle) = self.signaling_thread.take() {
            if let Err(e) = handle.join() {
                error!("シグナリングスレッドの終了に失敗: {:?}", e);
            }
        }
        
        // ワーカースレッドの終了を待機
        if let Some(handle) = self.worker_thread.take() {
            if let Err(e) = handle.join() {
                error!("ワーカースレッドの終了に失敗: {:?}", e);
            }
        }
        
        // すべてのクライアントセッションを終了
        let mut sessions = self.client_sessions.lock().unwrap();
        sessions.clear();
        
        self.thread_control = None;
        
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        match self.running.lock() {
            Ok(guard) => *guard,
            Err(_) => false,
        }
    }
    
    fn connected_clients(&self) -> usize {
        match self.client_sessions.lock() {
            Ok(sessions) => sessions.len(),
            Err(_) => 0,
        }
    }
    
    fn get_address(&self) -> SocketAddr {
        self.server_addr.unwrap_or_else(|| "0.0.0.0:0".parse().unwrap())
    }
}

#[cfg(not(feature = "webrtc-support"))]
impl WebRtcServer {
    /// 新しいWebRTCサーバーを作成
    pub fn new(
        _config: ServerConfig,
        _screen_capture: Arc<Mutex<ScreenCapture>>,
        _input_handler: Arc<Mutex<InputHandler>>,
    ) -> Result<Self, NetworkError> {
        Err(NetworkError::Other("WebRTCサポートが有効になっていません".to_string()))
    }
}

#[cfg(not(feature = "webrtc-support"))]
impl NetworkServer for WebRtcServer {
    fn start(&mut self) -> Result<(), NetworkError> {
        Err(NetworkError::Other("WebRTCサポートが有効になっていません".to_string()))
    }
    
    fn stop(&mut self) -> Result<(), NetworkError> {
        Err(NetworkError::Other("WebRTCサポートが有効になっていません".to_string()))
    }
    
    fn is_running(&self) -> bool {
        false
    }
    
    fn connected_clients(&self) -> usize {
        0
    }
    
    fn get_address(&self) -> SocketAddr {
        "0.0.0.0:0".parse().unwrap()
    }
}

/// WebRTCクライアント接続
#[cfg(feature = "webrtc-support")]
struct WebRtcClientConnection {
    /// Tokio ランタイム
    runtime: Arc<Runtime>,
    /// WebRTC ピア接続
    peer_connection: Arc<RTCPeerConnection>,
    /// メッセージ受信キュー
    message_rx: Option<tokio::sync::mpsc::Receiver<Vec<u8>>>,
    /// メッセージバッファ
    message_buffer: Vec<u8>,
}

#[cfg(feature = "webrtc-support")]
impl ClientConnection for WebRtcClientConnection {
    fn send(&mut self, response: &Response) -> Result<(), NetworkError> {
        // レスポンスをJSON形式にシリアライズ
        let json_data = serde_json::to_string(response)
            .map_err(|e| NetworkError::ProtocolError(format!("JSONシリアライズエラー: {}", e)))?;
        
        // データを送信
        self.send_raw(json_data.as_bytes())
    }
    
    fn send_raw(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        // 既存のデータチャネルを使用
        self.runtime.block_on(async {
            if let Some(dc) = self.peer_connection.data_channels().await.get(0) {
                dc.send_text(String::from_utf8_lossy(data).to_string()).await
                    .map_err(|e| NetworkError::CommunicationError(format!("データ送信エラー: {}", e)))
            } else {
                Err(NetworkError::ConnectionError("データチャネルがありません".to_string()))
            }
        })
    }
    
    fn receive(&mut self) -> Result<Command, NetworkError> {
        // メッセージキューからデータを受信
        if let Some(rx) = &mut self.message_rx {
            let data = self.runtime.block_on(async {
                // タイムアウト付きで受信
                match tokio::time::timeout(Duration::from_secs(30), rx.recv()).await {
                    Ok(Some(data)) => Ok(data),
                    Ok(None) => Err(NetworkError::ConnectionError("接続が閉じられました".to_string())),
                    Err(_) => Err(NetworkError::TimeoutError("受信タイムアウト".to_string())),
                }
            })?;
            
            // JSONからデシリアライズ
            serde_json::from_slice(&data)
                .map_err(|e| NetworkError::ProtocolError(format!("JSONデシリアライズエラー: {}", e)))
        } else {
            Err(NetworkError::ConnectionError("受信チャネルがありません".to_string()))
        }
    }
    
    fn set_timeout(&mut self, _duration: Duration) -> Result<(), NetworkError> {
        // WebRTC接続ではタイムアウトの設定は不要
        // (receive関数内で設定しているため)
        Ok(())
    }
    
    fn close(&mut self) -> Result<(), NetworkError> {
        // ピア接続をクローズ
        self.runtime.block_on(async {
            let _ = self.peer_connection.close().await;
        });
        
        self.message_rx = None;
        
        Ok(())
    }
}