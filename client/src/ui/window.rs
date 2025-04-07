//! メインウィンドウ
//!
//! アプリケーションのメインウィンドウを実装します。

use super::{ControlPanel, SettingsPanel, AppSettings, AppState, DisplayMode, PerformanceInfo, Styles};
use crate::display::{DisplayRenderer, ImageData, ImageFormat};
use crate::input::{InputEventHandler, InputEvent, MouseButton};
use crate::network::{NetworkClient, ConnectionInfo, TcpClient, WebSocketClient, WebRtcClient, Command, Response};

use eframe::{egui, epi};
use egui::{vec2, Rect, Ui, Key, Pos2, Context, ColorImage};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// メインウィンドウ
pub struct MainWindow {
    /// UI状態
    state: AppState,
    /// 設定パネル
    settings_panel: SettingsPanel,
    /// コントロールパネル
    control_panel: ControlPanel,
    /// 画面レンダラー
    renderer: DisplayRenderer,
    /// 入力イベントハンドラ
    input_handler: InputEventHandler,
    /// ネットワーククライアント
    client: Option<Box<dyn NetworkClient>>,
    /// 自動更新の間隔 (ミリ秒)
    update_interval: u64,
    /// 自動更新が有効かどうか
    auto_update: bool,
    /// 次回の自動更新の時間
    next_update: Option<Instant>,
    /// 最後の通信時刻
    last_communication: Option<Instant>,
    /// スタイル設定
    styles: Styles,
    /// パフォーマンス測定用の前回のフレーム時間
    last_frame_time: Option<Instant>,
    /// 直近のフレーム時間 (パフォーマンス計測用)
    frame_times: Vec<f32>,
    /// 設定パネルの表示状態
    show_settings: bool,
    /// コントロールパネルの表示状態
    show_controls: bool,
    /// 接続ダイアログの表示状態
    show_connection_dialog: bool,
    /// 接続情報
    connection_info: ConnectionInfo,
    /// エラーメッセージ
    error_message: Option<String>,
    /// ネットワークスレッド
    network_thread: Option<thread::JoinHandle<()>>,
    /// ネットワークスレッド通信チャネル
    thread_comm: Option<Arc<ThreadCommunication>>,
}

/// スレッド間通信
struct ThreadCommunication {
    /// 最新の画像データ
    image_data: RwLock<Option<ImageData>>,
    /// 実行中かどうか
    running: std::sync::atomic::AtomicBool,
    /// レイテンシー
    latency: std::sync::atomic::AtomicU64,
    /// コマンドキュー
    command_queue: Mutex<Vec<Command>>,
    /// レスポンスキュー
    response_queue: Mutex<Vec<Response>>,
}

impl MainWindow {
    /// 新しいメインウィンドウを作成
    pub fn new(cc: &eframe::CreationContext) -> Self {
        // デフォルトのeGUIスタイルをカスタマイズ
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace)),
            (egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Small, egui::FontId::new(12.0, egui::FontFamily::Proportional)),
        ].into();
        cc.egui_ctx.set_style(style);

        Self {
            state: AppState::default(),
            settings_panel: SettingsPanel::new(),
            control_panel: ControlPanel::new(),
            renderer: DisplayRenderer::new(cc.egui_ctx.clone()),
            input_handler: InputEventHandler::new(),
            client: None,
            update_interval: 200,
            auto_update: false,
            next_update: None,
            last_communication: None,
            styles: Styles::default(),
            last_frame_time: None,
            frame_times: Vec::with_capacity(60),
            show_settings: false,
            show_controls: true,
            show_connection_dialog: false,
            connection_info: ConnectionInfo::default(),
            error_message: None,
            network_thread: None,
            thread_comm: None,
        }
    }

    /// 接続ダイアログを表示
    fn show_connection_dialog(&mut self, ui: &mut Ui) {
        egui::Window::new("接続")
            .fixed_size(vec2(400.0, 200.0))
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.heading("サーバーに接続");
                
                ui.add_space(10.0);
                
                // ホスト名/IPアドレス
                ui.horizontal(|ui| {
                    ui.label("ホスト:");
                    ui.text_edit_singleline(&mut self.connection_info.host);
                });
                
                // ポート番号
                ui.horizontal(|ui| {
                    ui.label("ポート:");
                    ui.add(egui::DragValue::new(&mut self.connection_info.port).speed(1.0));
                });
                
                // 認証情報
                ui.collapsing("認証設定", |ui| {
                    ui.checkbox(&mut self.connection_info.use_auth, "認証を使用");
                    
                    if self.connection_info.use_auth {
                        ui.horizontal(|ui| {
                            ui.label("ユーザー名:");
                            let username = self.connection_info.username.get_or_insert_with(String::new);
                            ui.text_edit_singleline(username);
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("パスワード:");
                            let password = self.connection_info.password.get_or_insert_with(String::new);
                            let mut password_display = "*".repeat(password.len());
                            if ui.text_edit_singleline(&mut password_display).changed() {
                                *password = password_display;
                            }
                        });
                    }
                });
                
                // 接続プロトコル選択
                ui.horizontal(|ui| {
                    ui.label("接続方式:");
                    ui.radio_value(&mut self.connection_info.protocol, "tcp", "TCP");
                    ui.radio_value(&mut self.connection_info.protocol, "websocket", "WebSocket");
                    ui.radio_value(&mut self.connection_info.protocol, "webrtc", "WebRTC");
                });
                
                ui.add_space(10.0);
                
                // エラーメッセージ表示
                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, error);
                }
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("キャンセル").clicked() {
                        self.show_connection_dialog = false;
                    }
                    
                    if ui.button("接続").clicked() {
                        self.connect();
                    }
                });
            });
    }

    /// サーバーに接続
    fn connect(&mut self) {
        self.error_message = None;

        // 既に接続中なら切断
        if self.client.is_some() {
            self.disconnect();
        }

        // 接続プロトコルに応じたクライアントを作成
        let client: Box<dyn NetworkClient> = match self.connection_info.protocol.as_str() {
            "tcp" => Box::new(TcpClient::new()),
            "websocket" => Box::new(WebSocketClient::new()),
            "webrtc" => {
                match WebRtcClient::new() {
                    Ok(client) => Box::new(client),
                    Err(e) => {
                        self.error_message = Some(format!("WebRTCクライアントの初期化に失敗しました: {}", e));
                        return;
                    }
                }
            },
            _ => {
                self.error_message = Some("不明な接続プロトコルです".to_string());
                return;
            }
        };

        self.client = Some(client);

        // スレッド間通信用構造体を作成
        let thread_comm = Arc::new(ThreadCommunication {
            image_data: RwLock::new(None),
            running: std::sync::atomic::AtomicBool::new(true),
            latency: std::sync::atomic::AtomicU64::new(0),
            command_queue: Mutex::new(Vec::new()),
            response_queue: Mutex::new(Vec::new()),
        });
        self.thread_comm = Some(thread_comm.clone());

        // 接続情報のクローン
        let conn_info = self.connection_info.clone();

        // 接続を別スレッドで実行
        let handle = thread::spawn(move || {
            let mut client = match conn_info.protocol.as_str() {
                "tcp" => TcpClient::new(),
                "websocket" => WebSocketClient::new(),
                "webrtc" => WebRtcClient::new().unwrap(),
                _ => return,
            };

            // 接続
            match client.connect(&conn_info) {
                Ok(_) => {
                    // 接続成功のレスポンスをキューに追加
                    let mut responses = thread_comm.response_queue.lock().unwrap();
                    responses.push(Response::ConnectionStatus { 
                        connected: true, 
                        message: format!("{}に接続しました", conn_info.host) 
                    });
                },
                Err(e) => {
                    // 接続失敗のレスポンスをキューに追加
                    let mut responses = thread_comm.response_queue.lock().unwrap();
                    responses.push(Response::ConnectionStatus { 
                        connected: false, 
                        message: format!("接続に失敗しました: {}", e) 
                    });
                    return;
                }
            }

            // 画質設定を送信
            let _ = client.send(Command::SetQuality { quality: 50 });

            // 通信ループ
            while thread_comm.running.load(std::sync::atomic::Ordering::Relaxed) {
                // コマンドキューからコマンドを取り出して送信
                let commands = {
                    let mut queue = thread_comm.command_queue.lock().unwrap();
                    if queue.is_empty() {
                        Vec::new()
                    } else {
                        let commands = queue.clone();
                        queue.clear();
                        commands
                    }
                };

                for cmd in commands {
                    if let Err(e) = client.send(cmd.clone()) {
                        let mut responses = thread_comm.response_queue.lock().unwrap();
                        responses.push(Response::Error { 
                            code: -1, 
                            message: format!("コマンド送信に失敗しました: {}", e) 
                        });
                    }
                }

                // スクリーンショットを定期的に要求
                if thread_comm.image_data.read().unwrap().is_none() || 
                   commands.iter().any(|cmd| matches!(cmd, Command::RequestScreenshot { .. })) {
                    
                    // レイテンシー計測開始
                    let start = Instant::now();
                    
                    // スクリーンショット要求を送信
                    match client.send(Command::RequestScreenshot { 
                        quality: Some(50), width: None, height: None, monitor: None 
                    }) {
                        Ok(_) => {
                            // レスポンスを待機
                            match client.receive() {
                                Ok(Response::ScreenshotData { data, format, width, height, timestamp }) => {
                                    // レイテンシーを記録
                                    let latency = start.elapsed().as_millis() as u64;
                                    thread_comm.latency.store(latency, std::sync::atomic::Ordering::Relaxed);
                                    
                                    // 画像データを保存
                                    let format = match format {
                                        crate::network::protocol::ImageFormat::JPEG => ImageFormat::JPEG,
                                        crate::network::protocol::ImageFormat::PNG => ImageFormat::PNG,
                                        crate::network::protocol::ImageFormat::WebP => ImageFormat::WebP,
                                        crate::network::protocol::ImageFormat::AVIF => ImageFormat::AVIF,
                                    };
                                    
                                    let image_data = ImageData {
                                        data,
                                        format,
                                        width,
                                        height,
                                        timestamp,
                                    };
                                    
                                    *thread_comm.image_data.write().unwrap() = Some(image_data);
                                },
                                Ok(other) => {
                                    let mut responses = thread_comm.response_queue.lock().unwrap();
                                    responses.push(other);
                                },
                                Err(e) => {
                                    let mut responses = thread_comm.response_queue.lock().unwrap();
                                    responses.push(Response::Error { 
                                        code: -1, 
                                        message: format!("レスポンス受信に失敗しました: {}", e) 
                                    });
                                }
                            }
                        },
                        Err(e) => {
                            let mut responses = thread_comm.response_queue.lock().unwrap();
                            responses.push(Response::Error { 
                                code: -1, 
                                message: format!("スクリーンショット要求に失敗しました: {}", e) 
                            });
                        }
                    }
                }

                // スリープで余分なCPU使用を抑える
                thread::sleep(Duration::from_millis(50));
            }

            // 切断
            let _ = client.disconnect();
        });

        self.network_thread = Some(handle);
        self.show_connection_dialog = false;
        self.auto_update = true;
        self.state.connected = true;
    }

    /// サーバーから切断
    fn disconnect(&mut self) {
        if let Some(thread_comm) = &self.thread_comm {
            // スレッドの終了フラグを設定
            thread_comm.running.store(false, std::sync::atomic::Ordering::Relaxed);
        }

        // スレッドの終了を待機
        if let Some(handle) = self.network_thread.take() {
            let _ = handle.join();
        }

        // クライアントをクリーンアップ
        self.client = None;
        self.thread_comm = None;
        self.state.connected = false;
        self.auto_update = false;
    }

    /// メインディスプレイを描画
    fn draw_main_display(&mut self, ui: &mut Ui) {
        let available_size = ui.available_size();
        let display_rect = Rect::from_min_size(ui.min_pos(), available_size);

        // 画像があれば表示
        if let Some(texture_id) = self.renderer.get_texture() {
            let img_size = self.renderer.get_image_size();
            
            // 表示モードに応じたリサイズ
            let show_size = match self.state.display_mode {
                DisplayMode::Window => {
                    let ratio = (available_size.x / img_size.x).min(available_size.y / img_size.y);
                    vec2(img_size.x * ratio, img_size.y * ratio)
                },
                DisplayMode::Fullscreen => available_size,
                DisplayMode::Scaled => {
                    let ratio = (available_size.x / img_size.x).min(available_size.y / img_size.y);
                    vec2(img_size.x * ratio, img_size.y * ratio)
                },
                DisplayMode::OneToOne => img_size,
            };
            
            // 画像が実際のサイズよりも小さい場合は中央に配置
            let pos = if show_size.x < available_size.x || show_size.y < available_size.y {
                let x = ui.min_pos().x + (available_size.x - show_size.x) / 2.0;
                let y = ui.min_pos().y + (available_size.y - show_size.y) / 2.0;
                Pos2::new(x, y)
            } else {
                ui.min_pos()
            };
            
            // 画像表示領域
            let image_rect = Rect::from_min_size(pos, show_size);
            
            // 画像を描画
            ui.painter().image(texture_id, image_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), egui::Color32::WHITE);
            
            // レンダラーに表示領域を設定（座標変換用）
            self.renderer.set_display_rect(image_rect);
            
            // インプット処理用にレスポンスエリアを設定
            let response = ui.allocate_rect(image_rect, egui::Sense::click_and_drag());
            
            // マウスイベント処理
            if self.state.connected {
                // マウスの現在位置
                if let Some(pos) = response.hover_pos() {
                    let screen_pos = self.renderer.convert_to_screen_pos(pos, image_rect.min, image_rect.size());
                    
                    // マウス移動イベント
                    if let Some(cmd) = self.input_handler.handle_mouse_move(screen_pos) {
                        self.send_command(cmd);
                    }
                }
                
                // クリックイベント
                if response.clicked() {
                    if let Some(cmd) = self.input_handler.handle_mouse_click(MouseButton::Left) {
                        self.send_command(cmd);
                    }
                }
                
                // 右クリック
                if response.secondary_clicked() {
                    if let Some(cmd) = self.input_handler.handle_mouse_click(MouseButton::Right) {
                        self.send_command(cmd);
                    }
                }
                
                // ドラッグイベント
                if response.dragged() {
                    if let Some(pos) = response.hover_pos() {
                        let screen_pos = self.renderer.convert_to_screen_pos(pos, image_rect.min, image_rect.size());
                        
                        // ドラッグ中のマウス移動
                        if let Some(cmd) = self.input_handler.handle_mouse_move(screen_pos) {
                            self.send_command(cmd);
                        }
                    }
                }
                
                // ドラッグ開始
                if response.drag_started() {
                    if let Some(cmd) = self.input_handler.handle_mouse_down(MouseButton::Left) {
                        self.send_command(cmd);
                    }
                }
                
                // ドラッグ終了
                if response.drag_released() {
                    if let Some(cmd) = self.input_handler.handle_mouse_up(MouseButton::Left) {
                        self.send_command(cmd);
                    }
                }
            }
        } else {
            // 画像がない場合は説明テキストを表示
            ui.painter().rect_filled(display_rect, 0.0, egui::Color32::from_rgb(40, 40, 40));
            
            if self.state.connected {
                // 接続中だがまだ画像がない場合
                ui.centered_and_justified(|ui| {
                    ui.label("スクリーンショットを読み込み中...");
                });
            } else {
                // 未接続の場合
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("サーバーに接続されていません");
                        ui.add_space(8.0);
                        if ui.button("接続").clicked() {
                            self.show_connection_dialog = true;
                        }
                    });
                });
            }
        }
    }

    /// コマンドを送信
    fn send_command(&mut self, command: Command) {
        if let Some(thread_comm) = &self.thread_comm {
            let mut queue = thread_comm.command_queue.lock().unwrap();
            queue.push(command);
        }
    }

    /// フレームごとの更新処理
    fn update_frame(&mut self, ctx: &Context) {
        // 現在時刻
        let now = Instant::now();

        // FPS計測
        if let Some(last_time) = self.last_frame_time {
            let frame_time = now.duration_since(last_time).as_secs_f32() * 1000.0; // ミリ秒
            self.frame_times.push(frame_time);
            
            // 最新の60フレームのみを保持
            if self.frame_times.len() > 60 {
                self.frame_times.remove(0);
            }
            
            // 平均フレーム時間とFPSを計算
            let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            let fps = 1000.0 / avg_frame_time;
            
            // パフォーマンス情報を更新
            self.state.performance.fps = fps;
            self.state.performance.frame_time = avg_frame_time;
        }
        self.last_frame_time = Some(now);

        // スレッド通信を処理
        if let Some(thread_comm) = &self.thread_comm {
            // 画像データの更新を確認
            if let Some(image_data) = thread_comm.image_data.read().unwrap().clone() {
                // レンダラーに画像を転送
                let _ = self.renderer.update_image(image_data.data.clone(), image_data.width, image_data.height);
                
                // レイテンシーを更新
                self.state.performance.latency = thread_comm.latency.load(std::sync::atomic::Ordering::Relaxed);
            }
            
            // レスポンスキューからレスポンスを処理
            let responses = {
                let mut queue = thread_comm.response_queue.lock().unwrap();
                let responses = queue.clone();
                queue.clear();
                responses
            };
            
            for response in responses {
                match response {
                    Response::ConnectionStatus { connected, message } => {
                        self.state.connected = connected;
                        if !connected {
                            self.error_message = Some(message);
                        }
                    },
                    Response::Error { code: _, message } => {
                        self.error_message = Some(message);
                    },
                    _ => {}
                }
            }
        }

        // 自動更新処理
        if self.auto_update && self.state.connected {
            let do_update = if let Some(next_update) = self.next_update {
                now >= next_update
            } else {
                true
            };
            
            if do_update {
                self.send_command(Command::RequestScreenshot { 
                    quality: Some(self.state.performance.quality), 
                    width: None, 
                    height: None, 
                    monitor: None 
                });
                
                self.next_update = Some(now + Duration::from_millis(self.update_interval));
            }
        }

        // 継続的な再描画要求
        ctx.request_repaint();
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut epi::Frame) {
        // フレーム更新処理
        self.update_frame(ctx);
        
        // 接続ダイアログ
        if self.show_connection_dialog {
            self.show_connection_dialog(&mut egui::Ui::__internal_new(ctx.clone()));
        }

        // 設定パネル
        if self.show_settings {
            egui::Window::new("設定")
                .collapsible(true)
                .resizable(true)
                .show(ctx, |ui| {
                    self.settings_panel.ui(ui, &mut self.state.settings);
                    
                    if ui.button("閉じる").clicked() {
                        self.show_settings = false;
                    }
                });
        }

        // トップバー
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.button("接続...").clicked() {
                        ui.close_menu();
                        self.show_connection_dialog = true;
                    }
                    
                    if ui.button("切断").clicked() {
                        ui.close_menu();
                        self.disconnect();
                    }
                    
                    ui.separator();
                    
                    if ui.button("終了").clicked() {
                        frame.quit();
                    }
                });
                
                ui.menu_button("表示", |ui| {
                    if ui.button("フルスクリーン").clicked() {
                        ui.close_menu();
                        self.state.display_mode = DisplayMode::Fullscreen;
                    }
                    
                    if ui.button("ウィンドウモード").clicked() {
                        ui.close_menu();
                        self.state.display_mode = DisplayMode::Window;
                    }
                    
                    if ui.button("1:1サイズ").clicked() {
                        ui.close_menu();
                        self.state.display_mode = DisplayMode::OneToOne;
                    }
                    
                    ui.separator();
                    
                    if ui.checkbox(&mut self.show_controls, "コントロールパネル").clicked() {
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("設定", |ui| {
                    if ui.button("環境設定...").clicked() {
                        ui.close_menu();
                        self.show_settings = true;
                    }
                    
                    ui.menu_button("画質設定", |ui| {
                        if ui.button("最高画質").clicked() {
                            ui.close_menu();
                            self.state.performance.quality = 90;
                        }
                        
                        if ui.button("高画質").clicked() {
                            ui.close_menu();
                            self.state.performance.quality = 70;
                        }
                        
                        if ui.button("標準").clicked() {
                            ui.close_menu();
                            self.state.performance.quality = 50;
                        }
                        
                        if ui.button("低画質").clicked() {
                            ui.close_menu();
                            self.state.performance.quality = 30;
                        }
                        
                        if ui.button("最低画質").clicked() {
                            ui.close_menu();
                            self.state.performance.quality = 10;
                        }
                    });
                });
                
                ui.menu_button("ヘルプ", |ui| {
                    if ui.button("バージョン情報").clicked() {
                        ui.close_menu();
                        // TODO: バージョン情報ダイアログ
                    }
                });
                
                // 右寄せの要素
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    // 接続状態表示
                    if self.state.connected {
                        ui.colored_label(egui::Color32::GREEN, "接続済み");
                    } else {
                        ui.colored_label(egui::Color32::RED, "未接続");
                    }
                    
                    // パフォーマンス情報
                    ui.label(format!("{:.1} FPS", self.state.performance.fps));
                    ui.label(format!("遅延: {}ms", self.state.performance.latency));
                });
            });
        });

        // コントロールパネル
        if self.show_controls {
            egui::SidePanel::right("controls_panel")
                .resizable(true)
                .default_width(200.0)
                .show(ctx, |ui| {
                    self.control_panel.ui(ui, &mut self.state);
                    
                    ui.separator();
                    
                    if !self.state.connected {
                        if ui.button("接続").clicked() {
                            self.show_connection_dialog = true;
                        }
                    } else {
                        if ui.button("切断").clicked() {
                            self.disconnect();
                        }
                    }
                    
                    ui.separator();
                    
                    ui.checkbox(&mut self.auto_update, "自動更新");
                    
                    if self.auto_update {
                        ui.add(egui::Slider::new(&mut self.update_interval, 50..=1000).text("更新間隔(ms)"));
                    }
                    
                    ui.add(egui::Slider::new(&mut self.state.performance.quality, 10..=90).text("画質"));
                    
                    if ui.button("画面更新").clicked() {
                        self.send_command(Command::RequestScreenshot { 
                            quality: Some(self.state.performance.quality), 
                            width: None, 
                            height: None, 
                            monitor: None 
                        });
                    }
                });
        }

        // メインパネル（画面表示領域）
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_main_display(ui);
        });

        // エラーメッセージ表示
        if let Some(error) = &self.error_message {
            egui::Window::new("エラー")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, error);
                    
                    if ui.button("閉じる").clicked() {
                        self.error_message = None;
                    }
                });
        }
    }
}

/// 接続情報の拡張
impl ConnectionInfo {
    pub fn use_auth(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }
    
    pub fn set_use_auth(&mut self, use_auth: bool) {
        if use_auth {
            if self.username.is_none() {
                self.username = Some(String::new());
            }
            if self.password.is_none() {
                self.password = Some(String::new());
            }
        } else {
            self.username = None;
            self.password = None;
        }
    }
}