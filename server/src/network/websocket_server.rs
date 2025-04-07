//! WebSocket サーバー実装
//!
//! WebSocket を使用してクライアントと通信するサーバーを実装します。
//! これにより、Webブラウザからの接続が可能になります。

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

// WebSocket関連
use tungstenite::{accept, WebSocket, Message};
use tungstenite::protocol::Role;

// TLS関連
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::io::prelude::*;

/// WebSocket サーバー
pub struct WebSocketServer {
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
    /// TLS Acceptor（TLS有効時）
    tls_acceptor: Option<TlsAcceptor>,
    /// サーバーアドレス
    server_addr: Option<SocketAddr>,
}

impl WebSocketServer {
    /// 新しいWebSocketサーバーを作成
    pub fn new(
        config: ServerConfig,
        screen_capture: Arc<Mutex<ScreenCapture>>,
        input_handler: Arc<Mutex<InputHandler>>,
    ) -> Result<Self, NetworkError> {
        // TLSが有効な場合はTLS Acceptorを作成
        let tls_acceptor = if config.use_tls {
            // 証明書と秘密鍵のパスを確認
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
                
            // 本来はここでPKCS#12形式のアイデンティティを作成する
            // このサンプルでは簡略化のため、暗号化されたPFXファイルを使用すると仮定
            let identity = Identity::from_pkcs12(&cert_data, "password")
                .map_err(|e| NetworkError::Other(format!("TLSアイデンティティの作成エラー: {}", e)))?;
                
            // TLS Acceptorを作成
            let tls_acceptor = TlsAcceptor::new(identity)
                .map_err(|e| NetworkError::Other(format!("TLS Acceptorの作成エラー: {}", e)))?;
                
            Some(tls_acceptor)
        } else {
            None
        };
        
        Ok(Self {
            config,
            screen_capture,
            input_handler,
            authenticator: Authenticator::new(),
            listener_thread: None,
            thread_control: None,
            client_sessions: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            tls_acceptor,
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
        tls_acceptor: Option<TlsAcceptor>,
    ) {
        // セッション情報を作成
        let session_id = uuid::Uuid::new_v4().to_string();
        let session_info = SessionInfo::new(session_id.clone(), client_addr.to_string());
        
        // WebSocket接続を確立
        thread::spawn(move || {
            info!("WebSocket接続受付: {}", client_addr);
            
            // TLS接続の場合はTLS handshakeを実行
            let websocket_result = if let Some(tls_acceptor) = tls_acceptor {
                // TLS接続をアクセプト
                match tls_acceptor.accept(stream) {
                    Ok(tls_stream) => {
                        // WebSocketハンドシェイク
                        accept(tls_stream).map(|ws| (ws, true))
                    },
                    Err(e) => {
                        error!("TLSハンドシェイクエラー: {}", e);
                        return;
                    }
                }
            } else {
                // 通常のWebSocketハンドシェイク
                accept(stream).map(|ws| (ws, false))
            };
            
            // WebSocket接続が確立できなかった場合は終了
            let (websocket, is_tls) = match websocket_result {
                Ok(ws) => ws,
                Err(e) => {
                    error!("WebSocketハンドシェイクエラー: {}", e);
                    return;
                }
            };
            
            info!("WebSocket接続確立: {} (TLS: {})", client_addr, is_tls);
            
            // 接続タイプに応じたクライアントセッションを作成
            let client_connection: Box<dyn ClientConnection> = if is_tls {
                Box::new(WebSocketTlsClientConnection { websocket })
            } else {
                Box::new(WebSocketClientConnection { websocket })
            };
            
            // クライアントセッションを作成
            let session = ClientSession::new(
                session_info.clone(),
                client_connection,
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
            
            // 接続処理ループ
            loop {
                // セッションを処理
                let mut session_lock = session.lock().unwrap();
                
                // セッションがアクティブでなければ終了
                if !session_lock.is_active() {
                    break;
                }
                
                // コマンドの受信と処理
                if let Err(e) = session_lock.receive_and_process() {
                    if let NetworkError::IoError(ref io_err) = e {
                        if io_err.kind() == std::io::ErrorKind::WouldBlock || 
                           io_err.kind() == std::io::ErrorKind::TimedOut {
                            // タイムアウト - セッションがまだアクティブかチェック
                            if session_lock.idle_time() > config.client_timeout {
                                warn!("クライアントタイムアウト: {}", client_addr);
                                break;
                            }
                        } else {
                            // その他のエラー
                            error!("WebSocket通信エラー: {}", e);
                            break;
                        }
                    } else {
                        // ネットワークエラー
                        error!("WebSocket接続エラー: {}", e);
                        break;
                    }
                }
                
                // 短いスリープを入れてCPU使用率を下げる
                drop(session_lock);
                thread::sleep(Duration::from_millis(1));
            }
            
            info!("WebSocket接続終了: {}", client_addr);
            
            // セッションリストから削除
            {
                let mut sessions = client_sessions.lock().unwrap();
                sessions.retain(|s| {
                    let s_lock = s.lock().unwrap();
                    s_lock.session_info().id != session_id
                });
            }
        });
    }
}

impl NetworkServer for WebSocketServer {
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
        let tls_acceptor = self.tls_acceptor.clone();
        
        // リスナースレッドを起動
        let listener_thread = thread::spawn(move || {
            // 実行中フラグをセット
            *running.lock().unwrap() = true;
            
            info!("WebSocketサーバー起動: {}", bind_addr);
            
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
                            tls_acceptor.clone(),
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
            
            info!("WebSocketサーバー停止");
            
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

/// WebSocketクライアント接続
struct WebSocketClientConnection {
    websocket: WebSocket<TcpStream>,
}

impl ClientConnection for WebSocketClientConnection {
    fn send(&mut self, response: &Response) -> Result<(), NetworkError> {
        // レスポンスをJSON形式にシリアライズ
        let json_data = serde_json::to_string(response)
            .map_err(|e| NetworkError::ProtocolError(format!("JSONシリアライズエラー: {}", e)))?;
        
        // WebSocketで送信
        self.websocket.write_message(Message::Text(json_data))
            .map_err(|e| NetworkError::CommunicationError(format!("WebSocket送信エラー: {}", e)))?;
        
        Ok(())
    }
    
    fn send_raw(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        // バイナリメッセージとして送信
        self.websocket.write_message(Message::Binary(data.to_vec()))
            .map_err(|e| NetworkError::CommunicationError(format!("WebSocket送信エラー: {}", e)))?;
        
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Command, NetworkError> {
        // 非ブロッキングモードに設定
        self.websocket.get_mut().set_nonblocking(true)?;
        
        // WebSocketからメッセージを受信
        let message = self.websocket.read_message()
            .map_err(|e| {
                match e {
                    tungstenite::Error::Io(io_err) => NetworkError::IoError(io_err),
                    _ => NetworkError::CommunicationError(format!("WebSocket受信エラー: {}", e)),
                }
            })?;
        
        // メッセージのタイプに応じて処理
        match message {
            Message::Text(text) => {
                // JSONからデシリアライズ
                serde_json::from_str(&text)
                    .map_err(|e| NetworkError::ProtocolError(format!("JSONデシリアライズエラー: {}", e)))
            },
            Message::Binary(data) => {
                // バイナリデータをJSONとしてデシリアライズ
                serde_json::from_slice(&data)
                    .map_err(|e| NetworkError::ProtocolError(format!("JSONデシリアライズエラー: {}", e)))
            },
            Message::Close(_) => {
                // クライアントが接続を閉じた
                Err(NetworkError::ConnectionError("WebSocket接続が閉じられました".to_string()))
            },
            _ => {
                // その他のメッセージタイプ（Ping/Pongなど）は無視
                Err(NetworkError::ProtocolError("不明なメッセージタイプです".to_string()))
            }
        }
    }
    
    fn set_timeout(&mut self, duration: Duration) -> Result<(), NetworkError> {
        // WebSocketの基盤となるTCPストリームにタイムアウトを設定
        self.websocket.get_mut().set_read_timeout(Some(duration))?;
        self.websocket.get_mut().set_write_timeout(Some(duration))?;
        
        Ok(())
    }
    
    fn close(&mut self) -> Result<(), NetworkError> {
        // WebSocket接続をクローズ
        let close_frame = tungstenite::protocol::CloseFrame {
            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
            reason: "接続終了".into(),
        };
        
        let _ = self.websocket.close(Some(close_frame));
        
        Ok(())
    }
}

/// WebSocket TLSクライアント接続
struct WebSocketTlsClientConnection {
    websocket: WebSocket<TlsStream<TcpStream>>,
}

impl ClientConnection for WebSocketTlsClientConnection {
    fn send(&mut self, response: &Response) -> Result<(), NetworkError> {
        // レスポンスをJSON形式にシリアライズ
        let json_data = serde_json::to_string(response)
            .map_err(|e| NetworkError::ProtocolError(format!("JSONシリアライズエラー: {}", e)))?;
        
        // WebSocketで送信
        self.websocket.write_message(Message::Text(json_data))
            .map_err(|e| NetworkError::CommunicationError(format!("WebSocket送信エラー: {}", e)))?;
        
        Ok(())
    }
    
    fn send_raw(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        // バイナリメッセージとして送信
        self.websocket.write_message(Message::Binary(data.to_vec()))
            .map_err(|e| NetworkError::CommunicationError(format!("WebSocket送信エラー: {}", e)))?;
        
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Command, NetworkError> {
        // 非ブロッキングモードに設定（TLSストリームでは直接設定できないため注意が必要）
        // 通常はTLS接続の基底となるTCPストリームにアクセスする必要がある
        
        // WebSocketからメッセージを受信
        let message = self.websocket.read_message()
            .map_err(|e| {
                match e {
                    tungstenite::Error::Io(io_err) => NetworkError::IoError(io_err),
                    _ => NetworkError::CommunicationError(format!("WebSocket受信エラー: {}", e)),
                }
            })?;
        
        // メッセージのタイプに応じて処理
        match message {
            Message::Text(text) => {
                // JSONからデシリアライズ
                serde_json::from_str(&text)
                    .map_err(|e| NetworkError::ProtocolError(format!("JSONデシリアライズエラー: {}", e)))
            },
            Message::Binary(data) => {
                // バイナリデータをJSONとしてデシリアライズ
                serde_json::from_slice(&data)
                    .map_err(|e| NetworkError::ProtocolError(format!("JSONデシリアライズエラー: {}", e)))
            },
            Message::Close(_) => {
                // クライアントが接続を閉じた
                Err(NetworkError::ConnectionError("WebSocket接続が閉じられました".to_string()))
            },
            _ => {
                // その他のメッセージタイプ（Ping/Pongなど）は無視
                Err(NetworkError::ProtocolError("不明なメッセージタイプです".to_string()))
            }
        }
    }
    
    fn set_timeout(&mut self, _duration: Duration) -> Result<(), NetworkError> {
        // TLS接続ではタイムアウトの設定は不要/直接設定できない
        // TLS接続のタイムアウト制御は基底のTCPソケットに依存する
        
        Ok(())
    }
    
    fn close(&mut self) -> Result<(), NetworkError> {
        // WebSocket接続をクローズ
        let close_frame = tungstenite::protocol::CloseFrame {
            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
            reason: "接続終了".into(),
        };
        
        let _ = self.websocket.close(Some(close_frame));
        
        Ok(())
    }
}