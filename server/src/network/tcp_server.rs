//! TCP サーバー実装
//!
//! TCP ソケットを使用してクライアントと通信するサーバーを実装します。
//! TLSとプレーンテキストの両方の接続をサポートします。

use super::{NetworkServer, NetworkError, ServerConfig, SessionInfo};
use super::session::ClientSession;
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

// TLS関連
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::io::prelude::*;

/// TCP Server
pub struct TcpServer {
    /// サーバー設定
    config: ServerConfig,
    /// スクリーンキャプチャー
    screen_capture: Arc<Mutex<ScreenCapture>>,
    /// 入力ハンドラー
    input_handler: Arc<Mutex<InputHandler>>,
    /// 認証ハンドラ
    authenticator: Authenticator,
    /// リスナースレッド
    listener_thread: Option<thread::JoinHandle<()>>,
    /// スレッド管理用チャネル
    thread_control: Option<mpsc::Sender<()>>,
    /// クライアントセッションリスト
    client_sessions: Arc<Mutex<Vec<Arc<Mutex<ClientSession>>>>>,
    /// 起動中フラグ
    running: Arc<Mutex<bool>>,
    /// サーバーアドレス
    server_addr: Option<SocketAddr>,
}

impl TcpServer {
    /// 新しいTCPサーバーを作成
    pub fn new(
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
    ) -> Result<Self, NetworkError> {
        Ok(Self {
            config,
            screen_capture,
            input_handler,
            authenticator: Authenticator::new(),
            listener_thread: None,
            thread_control: None,
            client_sessions: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            server_addr: None,
        })
    }
    
    /// クライアント接続を処理
    fn handle_client(
        stream: TcpStream,
        client_addr: SocketAddr,
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
        authenticator: Arc<Authenticator>,
        client_sessions: Arc<Mutex<Vec<Arc<Mutex<ClientSession>>>>>,
    ) {
        // セッション情報を作成
        let session_id = uuid::Uuid::new_v4().to_string();
        let session_info = SessionInfo::new(session_id.clone(), client_addr.to_string());
        
        // TCP通信用のクライアントセッションを作成
        let session = ClientSession::new(
            session_info.clone(),
            Box::new(TcpClientConnection::new(stream.try_clone().unwrap())),
            screen_capture.clone(),
            input_handler.clone(),
            authenticator.clone(),
            config.clone(),
        );
        
        let session = Arc::new(Mutex::new(session));
        
        // セッションリストに追加
        {
            let mut sessions = client_sessions.lock().unwrap();
            sessions.push(session.clone());
        }
        
        // TCP接続を非同期で処理
        thread::spawn(move || {
            info!("クライアント接続: {}", client_addr);
            
            let mut buffer = [0; 4096];
            let mut stream = stream;
            
            // 接続タイムアウトを設定
            let _ = stream.set_read_timeout(Some(Duration::from_secs(config.client_timeout)));
            let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
            
            // TCP_NODELAY設定
            let _ = stream.set_nodelay(true);
            
            // ソケットバッファサイズを設定
            let _ = stream.set_recv_buffer_size(1024 * 1024 * 10); // 10MB
            let _ = stream.set_send_buffer_size(1024 * 1024);      // 1MB
            
            loop {
                match stream.read(&mut buffer) {
                    Ok(0) => {
                        // 接続が閉じられた
                        info!("クライアント切断: {}", client_addr);
                        break;
                    },
                    Ok(bytes_read) => {
                        // 受信データをセッションに渡す
                        let mut session_lock = session.lock().unwrap();
                        let data = &buffer[..bytes_read];
                        
                        if let Err(e) = session_lock.process_data(data) {
                            error!("データ処理エラー: {}", e);
                            break;
                        }
                        
                        if !session_lock.is_active() {
                            // セッションが終了した
                            break;
                        }
                    },
                    Err(e) => {
                        // エラー発生
                        if e.kind() == std::io::ErrorKind::WouldBlock || 
                           e.kind() == std::io::ErrorKind::TimedOut {
                            // タイムアウト - セッションがまだアクティブかチェック
                            let session_lock = session.lock().unwrap();
                            if session_lock.idle_time() > config.client_timeout {
                                warn!("クライアントタイムアウト: {}", client_addr);
                                break;
                            }
                        } else {
                            // その他のエラー
                            error!("クライアント接続エラー: {}", e);
                            break;
                        }
                    }
                }
                
                // 短いスリープを入れてCPU使用率を下げる
                thread::sleep(Duration::from_millis(1));
            }
            
            // セッションリストから削除
            {
                let mut sessions = client_sessions.lock().unwrap();
                sessions.retain(|s| {
                    let s_lock = s.lock().unwrap();
                    s_lock.session_info().id != session_id
                });
            }
            
            info!("クライアントスレッド終了: {}", client_addr);
        });
    }
}

impl NetworkServer for TcpServer {
    fn start(&mut self) -> Result<(), NetworkError> {
        // 既に起動していれば何もしない
        if self.is_running() {
            return Ok(());
        }
        
        // サーバーソケットのバインドアドレス
        let bind_addr = format!("{}:{}", self.config.bind_address, self.config.port);
        
        // TCPリスナーを作成
        let listener = TcpListener::bind(&bind_addr)
            .map_err(|e| NetworkError::IoError(e))?;
        
        // 非ブロッキングモードに設定
        listener.set_nonblocking(true)
            .map_err(|e| NetworkError::IoError(e))?;
        
        // サーバーアドレスを保存
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
        
        // リスナースレッドを起動
        let listener_thread = thread::spawn(move || {
            // 実行中フラグをセット
            *running.lock().unwrap() = true;
            
            info!("TCPサーバー起動: {}", bind_addr);
            
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
                        
                        // クライアント処理関数を呼び出し
                        Self::handle_client(
                            stream,
                            addr,
                            config.clone(),
                            screen_capture.clone(),
                            input_handler.clone(),
                            authenticator.clone(),
                            client_sessions.clone(),
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
            
            info!("TCPサーバー停止");
            
            // 実行中フラグをクリア
            *running.lock().unwrap() = false;
        });
        
        self.listener_thread = Some(listener_thread);
        
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
        
        // リスナースレッドの終了を待機
        if let Some(handle) = self.listener_thread.take() {
            if let Err(e) = handle.join() {
                error!("リスナースレッドの終了に失敗: {:?}", e);
            }
        }
        
        // すべてのクライアントセッションを終了
        let mut sessions = self.client_sessions.lock().unwrap();
        sessions.clear();
        
        self.thread_control = None;
        self.server_addr = None;
        
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        match self.running.lock() {
            Ok(running) => *running,
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
        self.server_addr.unwrap_or_else(|| {
            SocketAddr::new(
                self.config.bind_address.parse().unwrap_or("127.0.0.1".parse().unwrap()),
                self.config.port
            )
        })
    }
}

/// TCPクライアント接続
pub struct TcpClientConnection {
    stream: TcpStream,
}

impl TcpClientConnection {
    /// 新しいTCPクライアント接続を作成
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
    
    /// コマンドをJSON文字列として送信
    fn send_json<T: serde::Serialize>(&mut self, data: &T) -> Result<(), NetworkError> {
        // JSONにシリアライズ
        let json_data = serde_json::to_string(data)
            .map_err(|e| NetworkError::ProtocolError(format!("JSONシリアライズエラー: {}", e)))?;
        
        // データ長を4バイトのプレフィックスとして送信
        let len = json_data.len() as u32;
        let len_bytes = len.to_be_bytes();
        
        self.stream.write_all(&len_bytes)?;
        self.stream.write_all(json_data.as_bytes())?;
        self.stream.flush()?;
        
        Ok(())
    }
    
    /// データを送信
    fn send_data(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        // データ長を4バイトのプレフィックスとして送信
        let len = data.len() as u32;
        let len_bytes = len.to_be_bytes();
        
        self.stream.write_all(&len_bytes)?;
        self.stream.write_all(data)?;
        self.stream.flush()?;
        
        Ok(())
    }
    
    /// メッセージの受信
    fn receive_message(&mut self) -> Result<Vec<u8>, NetworkError> {
        // メッセージ長のプレフィックスを読み取る
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        // メッセージ本体を読み取る
        let mut buffer = vec![0u8; len];
        self.stream.read_exact(&mut buffer)?;
        
        Ok(buffer)
    }
}

/// トレイト実装：クライアント接続
impl super::session::ClientConnection for TcpClientConnection {
    fn send(&mut self, response: &Response) -> Result<(), NetworkError> {
        self.send_json(response)
    }
    
    fn send_raw(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        self.send_data(data)
    }
    
    fn receive(&mut self) -> Result<Command, NetworkError> {
        let data = self.receive_message()?;
        
        // JSONからデシリアライズ
        serde_json::from_slice(&data)
            .map_err(|e| NetworkError::ProtocolError(format!("JSONデシリアライズエラー: {}", e)))
    }
    
    fn set_timeout(&mut self, duration: Duration) -> Result<(), NetworkError> {
        self.stream.set_read_timeout(Some(duration))?;
        self.stream.set_write_timeout(Some(duration))?;
        Ok(())
    }
    
    fn close(&mut self) -> Result<(), NetworkError> {
        // TCPコネクションをシャットダウン
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        Ok(())
    }
}

/// TLS TCP サーバー
pub struct TlsTcpServer {
    /// 内部TCP Server
    inner: TcpServer,
    /// TLS Acceptor
    tls_acceptor: TlsAcceptor,
}

impl TlsTcpServer {
    /// 新しいTLS TCPサーバーを作成
    pub fn new(
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
    ) -> Result<Self, NetworkError> {
        let inner = TcpServer::new(config.clone(), screen_capture, input_handler)?;
        
        // TLS証明書と秘密鍵からIDを作成
        let cert_path = config.tls_cert_path.as_ref()
            .ok_or_else(|| NetworkError::Other("TLS証明書パスが指定されていません".to_string()))?;
            
        let key_path = config.tls_key_path.as_ref()
            .ok_or_else(|| NetworkError::Other("TLS秘密鍵パスが指定されていません".to_string()))?;
            
        // PFXファイルを読み込む（または証明書と秘密鍵からIdentityを作成）
        let mut cert_file = File::open(cert_path)
            .map_err(|e| NetworkError::IoError(e))?;
            
        let mut cert_data = Vec::new();
        cert_file.read_to_end(&mut cert_data)
            .map_err(|e| NetworkError::IoError(e))?;
            
        let mut key_file = File::open(key_path)
            .map_err(|e| NetworkError::IoError(e))?;
            
        let mut key_data = Vec::new();
        key_file.read_to_end(&mut key_data)
            .map_err(|e| NetworkError::IoError(e))?;
            
        // 本来はここでPKCS#12形式のアイデンティティを作成する
        // このサンプルでは簡略化のため、暗号化されたPFXファイルを使用すると仮定
        let identity = Identity::from_pkcs12(&cert_data, "password")
            .map_err(|e| NetworkError::Other(format!("TLSアイデンティティの作成エラー: {}", e)))?;
            
        // TLS Acceptorを作成
        let tls_acceptor = TlsAcceptor::new(identity)
            .map_err(|e| NetworkError::Other(format!("TLS Acceptorの作成エラー: {}", e)))?;
            
        Ok(Self {
            inner,
            tls_acceptor,
        })
    }
}

impl NetworkServer for TlsTcpServer {
    fn start(&mut self) -> Result<(), NetworkError> {
        self.inner.start()
    }
    
    fn stop(&mut self) -> Result<(), NetworkError> {
        self.inner.stop()
    }
    
    fn is_running(&self) -> bool {
        self.inner.is_running()
    }
    
    fn connected_clients(&self) -> usize {
        self.inner.connected_clients()
    }
    
    fn get_address(&self) -> SocketAddr {
        self.inner.get_address()
    }
}

/// TLS TCP クライアント接続
pub struct TlsClientConnection {
    stream: TlsStream<TcpStream>,
}

impl TlsClientConnection {
    /// 新しいTLS TCP接続を作成
    pub fn new(stream: TlsStream<TcpStream>) -> Self {
        Self { stream }
    }
    
    /// コマンドをJSON文字列として送信
    fn send_json<T: serde::Serialize>(&mut self, data: &T) -> Result<(), NetworkError> {
        // JSONにシリアライズ
        let json_data = serde_json::to_string(data)
            .map_err(|e| NetworkError::ProtocolError(format!("JSONシリアライズエラー: {}", e)))?;
        
        // データ長を4バイトのプレフィックスとして送信
        let len = json_data.len() as u32;
        let len_bytes = len.to_be_bytes();
        
        self.stream.write_all(&len_bytes)?;
        self.stream.write_all(json_data.as_bytes())?;
        self.stream.flush()?;
        
        Ok(())
    }
    
    /// データを送信
    fn send_data(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        // データ長を4バイトのプレフィックスとして送信
        let len = data.len() as u32;
        let len_bytes = len.to_be_bytes();
        
        self.stream.write_all(&len_bytes)?;
        self.stream.write_all(data)?;
        self.stream.flush()?;
        
        Ok(())
    }
    
    /// メッセージの受信
    fn receive_message(&mut self) -> Result<Vec<u8>, NetworkError> {
        // メッセージ長のプレフィックスを読み取る
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        // メッセージ本体を読み取る
        let mut buffer = vec![0u8; len];
        self.stream.read_exact(&mut buffer)?;
        
        Ok(buffer)
    }
}

impl super::session::ClientConnection for TlsClientConnection {
    fn send(&mut self, response: &Response) -> Result<(), NetworkError> {
        self.send_json(response)
    }
    
    fn send_raw(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        self.send_data(data)
    }
    
    fn receive(&mut self) -> Result<Command, NetworkError> {
        let data = self.receive_message()?;
        
        // JSONからデシリアライズ
        serde_json::from_slice(&data)
            .map_err(|e| NetworkError::ProtocolError(format!("JSONデシリアライズエラー: {}", e)))
    }
    
    fn set_timeout(&mut self, duration: Duration) -> Result<(), NetworkError> {
        // TlsStreamに対する直接のタイムアウト設定はできないため、
        // 内部のTcpStreamに設定する
        let tcp_stream = self.stream.get_ref();
        tcp_stream.set_read_timeout(Some(duration))?;
        tcp_stream.set_write_timeout(Some(duration))?;
        Ok(())
    }
    
    fn close(&mut self) -> Result<(), NetworkError> {
        // TLS接続をシャットダウン
        let _ = self.stream.shutdown();
        Ok(())
    }
}