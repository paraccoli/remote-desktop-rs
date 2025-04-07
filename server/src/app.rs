//! サーバーアプリケーション
//!
//! リモートデスクトップサーバーのメインアプリケーションクラスを実装します。

use crate::capture::ScreenCapture;
use crate::input::InputHandler;
use crate::network::{NetworkServer, ServerConfig, ServerFactory, NetworkError};
use crate::ui::{ServerSettings, ServerState, StatusInfo, TrayHandler};

use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use std::path::{Path, PathBuf};
use std::fs;
use log::{info, warn, error, debug};

/// アプリケーション状態
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AppState {
    /// 初期化中
    Initializing,
    /// 準備完了
    Ready,
    /// サーバー起動中
    Starting,
    /// サーバー実行中
    Running,
    /// サーバー停止中
    Stopping,
    /// 終了中
    ShuttingDown,
    /// エラー発生
    Error,
}

/// アプリケーション
pub struct App {
    /// アプリケーション状態
    state: AppState,
    /// サーバー設定
    settings: ServerSettings,
    /// スクリーンキャプチャー
    screen_capture: Arc<Mutex<ScreenCapture>>,
    /// 入力ハンドラ
    input_handler: Arc<Mutex<InputHandler>>,
    /// ネットワークサーバー
    server: Option<Box<dyn NetworkServer + Send>>,
    /// サーバー状態
    server_state: Arc<Mutex<ServerState>>,
    /// ステータス情報
    status_info: Arc<Mutex<StatusInfo>>,
    /// システムトレイハンドラ
    tray_handler: Option<TrayHandler>,
    /// 設定ファイルパス
    settings_path: PathBuf,
    /// アプリケーション開始時刻
    start_time: Instant,
    /// トラフィック統計
    stats: TrafficStats,
    /// 終了コマンド受信チャネル
    exit_receiver: Option<mpsc::Receiver<()>>,
    /// 終了コマンド送信チャネル
    exit_sender: Option<mpsc::Sender<()>>,
}

/// トラフィック統計
struct TrafficStats {
    /// 送信バイト数
    sent_bytes: u64,
    /// 受信バイト数
    received_bytes: u64,
    /// 最大送信速度（バイト/秒）
    max_send_rate: u64,
    /// 最大受信速度（バイト/秒）
    max_receive_rate: u64,
    /// 前回の測定時刻
    last_measurement: Instant,
    /// 現在のウィンドウでの送信バイト数
    current_window_sent: u64,
    /// 現在のウィンドウでの受信バイト数
    current_window_received: u64,
}

impl TrafficStats {
    /// 新しいトラフィック統計を作成
    fn new() -> Self {
        Self {
            sent_bytes: 0,
            received_bytes: 0,
            max_send_rate: 0,
            max_receive_rate: 0,
            last_measurement: Instant::now(),
            current_window_sent: 0,
            current_window_received: 0,
        }
    }
    
    /// 送信バイト数を追加
    fn add_sent(&mut self, bytes: u64) {
        self.sent_bytes += bytes;
        self.current_window_sent += bytes;
        self.update_rates();
    }
    
    /// 受信バイト数を追加
    fn add_received(&mut self, bytes: u64) {
        self.received_bytes += bytes;
        self.current_window_received += bytes;
        self.update_rates();
    }
    
    /// レート情報を更新
    fn update_rates(&mut self) {
        let elapsed = self.last_measurement.elapsed();
        
        // 1秒以上経過していたらレート計算
        if elapsed >= Duration::from_secs(1) {
            let elapsed_secs = elapsed.as_secs_f64();
            
            // 送信レート（バイト/秒）
            let send_rate = (self.current_window_sent as f64 / elapsed_secs) as u64;
            if send_rate > self.max_send_rate {
                self.max_send_rate = send_rate;
            }
            
            // 受信レート（バイト/秒）
            let receive_rate = (self.current_window_received as f64 / elapsed_secs) as u64;
            if receive_rate > self.max_receive_rate {
                self.max_receive_rate = receive_rate;
            }
            
            // 次の測定のためにリセット
            self.last_measurement = Instant::now();
            self.current_window_sent = 0;
            self.current_window_received = 0;
        }
    }
    
    /// 送信バイト数を取得
    fn get_sent_bytes(&self) -> u64 {
        self.sent_bytes
    }
    
    /// 受信バイト数を取得
    fn get_received_bytes(&self) -> u64 {
        self.received_bytes
    }
    
    /// 最大送信レートを取得（バイト/秒）
    fn get_max_send_rate(&self) -> u64 {
        self.max_send_rate
    }
    
    /// 最大受信レートを取得（バイト/秒）
    fn get_max_receive_rate(&self) -> u64 {
        self.max_receive_rate
    }
}

impl App {
    /// 新しいアプリケーションを作成
    pub fn new() -> Result<Self, String> {
        // 設定ファイルパスを取得
        let settings_path = Self::get_settings_path();
        
        // 設定を読み込み
        let settings = ServerSettings::load(&settings_path)
            .map_err(|e| format!("設定の読み込みに失敗しました: {}", e))?;
        
        // キャプチャーを初期化
        let screen_capture = ScreenCapture::new()
            .map_err(|e| format!("スクリーンキャプチャの初期化に失敗しました: {}", e))?;
        
        // 入力ハンドラを初期化
        let input_handler = InputHandler::new()
            .map_err(|e| format!("入力ハンドラの初期化に失敗しました: {}", e))?;
        
        // 終了用チャネルを作成
        let (exit_sender, exit_receiver) = mpsc::channel();
        
        // サーバー状態を初期化
        let server_state = ServerState {
            running: false,
            connected_clients: 0,
            server_address: settings.network.bind_address.clone(),
            server_port: settings.network.port,
            tls_enabled: settings.network.use_tls,
            sessions: Vec::new(),
        };
        
        // ステータス情報を初期化
        let status_info = StatusInfo::default();
        
        Ok(Self {
            state: AppState::Initializing,
            settings,
            screen_capture: Arc::new(Mutex::new(screen_capture)),
            input_handler: Arc::new(Mutex::new(input_handler)),
            server: None,
            server_state: Arc::new(Mutex::new(server_state)),
            status_info: Arc::new(Mutex::new(status_info)),
            tray_handler: None,
            settings_path,
            start_time: Instant::now(),
            stats: TrafficStats::new(),
            exit_receiver: Some(exit_receiver),
            exit_sender: Some(exit_sender),
        })
    }
    
    /// アプリケーションを初期化
    pub fn initialize(&mut self) -> Result<(), String> {
        info!("リモートデスクトップサーバーを初期化中...");
        
        // システムトレイを初期化
        #[cfg(feature = "system-tray")]
        {
            let mut tray_handler = TrayHandler::new(self.server_state.clone());
            tray_handler.initialize()?;
            
            // 設定ダイアログを開くコールバックを設定
            let exit_sender = self.exit_sender.as_ref().unwrap().clone();
            tray_handler.set_settings_callback(move || {
                // 実際のアプリケーションでは設定ダイアログを開く処理をここに記述
                println!("設定ダイアログを開きます");
            });
            
            self.tray_handler = Some(tray_handler);
        }
        
        // キャプチャオプションを設定
        if let Some(monitor_index) = self.settings.capture.monitor_index {
            let mut screen_capture = self.screen_capture.lock().unwrap();
            let _ = screen_capture.select_monitor(monitor_index);
        }
        
        self.state = AppState::Ready;
        info!("リモートデスクトップサーバーの初期化が完了しました");
        
        Ok(())
    }
    
    /// サーバーを起動
    pub fn start_server(&mut self) -> Result<(), String> {
        if self.state == AppState::Running {
            return Ok(());
        }
        
        info!("リモートデスクトップサーバーを起動中...");
        self.state = AppState::Starting;
        
        // サーバー設定を構築
        let server_config = ServerConfig {
            bind_address: self.settings.network.bind_address.clone(),
            port: self.settings.network.port,
            use_tls: self.settings.network.use_tls,
            tls_cert_path: self.settings.network.tls_cert_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            tls_key_path: self.settings.network.tls_key_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            require_auth: self.settings.security.require_auth,
            max_connections: self.settings.network.max_connections,
            client_timeout: self.settings.network.client_timeout,
            keep_alive_interval: self.settings.network.keep_alive_interval,
        };
        
        // サーバーを選択して作成
        let server: Box<dyn NetworkServer + Send> = if self.settings.network.enable_webrtc {
            #[cfg(feature = "webrtc-support")]
            {
                ServerFactory::create_webrtc_server(
                    server_config,
                    self.screen_capture.clone(),
                    self.input_handler.clone(),
                )
                .map_err(|e| format!("WebRTCサーバーの作成に失敗しました: {}", e))?
            }
            #[cfg(not(feature = "webrtc-support"))]
            {
                return Err("WebRTCサポートが有効になっていません".to_string());
            }
        } else if self.settings.network.enable_websocket {
            ServerFactory::create_websocket_server(
                server_config,
                self.screen_capture.clone(),
                self.input_handler.clone(),
            )
            .map_err(|e| format!("WebSocketサーバーの作成に失敗しました: {}", e))?
        } else {
            ServerFactory::create_server(
                server_config,
                self.screen_capture.clone(),
                self.input_handler.clone(),
            )
            .map_err(|e| format!("TCPサーバーの作成に失敗しました: {}", e))?
        };
        
        // サーバーを起動
        server.start()
            .map_err(|e| format!("サーバーの起動に失敗しました: {}", e))?;
        
        // サーバーアドレスを取得
        let server_addr = server.get_address();
        
        // サーバー状態を更新
        {
            let mut state = self.server_state.lock().unwrap();
            state.running = true;
            state.server_address = server_addr.ip().to_string();
            state.server_port = server_addr.port();
        }
        
        // トレイ通知を送信
        if let Some(tray) = &mut self.tray_handler {
            tray.update_status(true, 0);
            tray.send_notification(
                "サーバー起動",
                &format!("リモートデスクトップサーバーが起動しました\nアドレス: {}:{}", 
                    server_addr.ip(), server_addr.port())
            );
            tray.update_tooltip(&format!("リモートデスクトップサーバー (実行中)\n{}:{}", 
                server_addr.ip(), server_addr.port()));
        }
        
        self.server = Some(server);
        self.state = AppState::Running;
        
        info!("リモートデスクトップサーバーが起動しました: {}:{}", server_addr.ip(), server_addr.port());
        
        Ok(())
    }
    
    /// サーバーを停止
    pub fn stop_server(&mut self) -> Result<(), String> {
        if self.state != AppState::Running {
            return Ok(());
        }
        
        info!("リモートデスクトップサーバーを停止中...");
        self.state = AppState::Stopping;
        
        if let Some(server) = self.server.take() {
            // サーバーを停止
            server.stop()
                .map_err(|e| format!("サーバーの停止に失敗しました: {}", e))?;
        }
        
        // サーバー状態を更新
        {
            let mut state = self.server_state.lock().unwrap();
            state.running = false;
            state.connected_clients = 0;
            state.sessions.clear();
        }
        
        // トレイ通知を送信
        if let Some(tray) = &mut self.tray_handler {
            tray.update_status(false, 0);
            tray.send_notification(
                "サーバー停止",
                "リモートデスクトップサーバーが停止しました"
            );
            tray.update_tooltip("リモートデスクトップサーバー (停止中)");
        }
        
        self.state = AppState::Ready;
        
        info!("リモートデスクトップサーバーが停止しました");
        
        Ok(())
    }
    
    /// 状態を更新
    pub fn update(&mut self) -> Result<(), String> {
        // 終了要求をチェック
        if let Some(rx) = &self.exit_receiver {
            if let Ok(_) = rx.try_recv() {
                self.state = AppState::ShuttingDown;
                return Ok(());
            }
        }
        
        // サーバーが実行中の場合
        if self.state == AppState::Running {
            if let Some(server) = &self.server {
                // 接続クライアント数を更新
                let connected_clients = server.connected_clients();
                
                {
                    let mut state = self.server_state.lock().unwrap();
                    if state.connected_clients != connected_clients {
                        state.connected_clients = connected_clients;
                        
                        // トレイハンドラに状態を更新
                        if let Some(tray) = &mut self.tray_handler {
                            tray.update_status(true, connected_clients);
                        }
                    }
                }
                
                // ステータス情報を更新
                {
                    let mut status = self.status_info.lock().unwrap();
                    status.uptime = self.start_time.elapsed().as_secs();
                    status.sent_bytes = self.stats.get_sent_bytes();
                    status.received_bytes = self.stats.get_received_bytes();
                    
                    // CPU・メモリ使用状況の取得はプラットフォーム依存
                    #[cfg(feature = "system-info")]
                    {
                        use sysinfo::{System, SystemExt, ProcessExt};
                        let mut system = System::new_all();
                        system.refresh_all();
                        
                        status.cpu_usage = system.global_processor_info().cpu_usage();
                        status.memory_usage = system.used_memory() / 1024 / 1024; // MBに変換
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// アプリケーションを実行
    pub fn run(&mut self) -> Result<(), String> {
        // アプリケーションを初期化
        self.initialize()?;
        
        // サーバーを起動
        self.start_server()?;
        
        // メインループ
        while self.state != AppState::ShuttingDown {
            // 更新処理
            self.update()?;
            
            // 短いスリープ
            thread::sleep(Duration::from_millis(100));
        }
        
        // サーバーを停止
        self.stop_server()?;
        
        info!("アプリケーションを終了します");
        
        Ok(())
    }
    
    /// 設定を保存
    pub fn save_settings(&self) -> Result<(), String> {
        self.settings.save(&self.settings_path)
    }
    
    /// 設定ファイルのパスを取得
    fn get_settings_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        
        let app_config_dir = config_dir.join("remote-desktop-rs").join("server");
        
        // ディレクトリが存在しない場合は作成
        if !app_config_dir.exists() {
            let _ = fs::create_dir_all(&app_config_dir);
        }
        
        app_config_dir.join("settings.json")
    }
    
    /// アプリケーション状態を取得
    pub fn get_state(&self) -> AppState {
        self.state
    }
    
    /// サーバーの状態を取得
    pub fn get_server_state(&self) -> Arc<Mutex<ServerState>> {
        self.server_state.clone()
    }
    
    /// ステータス情報を取得
    pub fn get_status_info(&self) -> Arc<Mutex<StatusInfo>> {
        self.status_info.clone()
    }
    
    /// 設定を取得
    pub fn get_settings(&self) -> &ServerSettings {
        &self.settings
    }
    
    /// 設定を更新
    pub fn update_settings(&mut self, settings: ServerSettings) -> Result<(), String> {
        // サーバーが実行中なら再起動が必要
        let need_restart = self.state == AppState::Running;
        
        if need_restart {
            self.stop_server()?;
        }
        
        // 設定を更新
        self.settings = settings;
        
        // 設定を保存
        self.save_settings()?;
        
        if need_restart {
            self.start_server()?;
        }
        
        Ok(())
    }
    
    /// 終了する
    pub fn shutdown(&mut self) -> Result<(), String> {
        if self.state == AppState::Running {
            self.stop_server()?;
        }
        
        // トレイハンドラをクリーンアップ
        self.tray_handler = None;
        
        self.state = AppState::ShuttingDown;
        
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}