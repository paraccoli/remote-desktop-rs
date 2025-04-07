//! システムトレイインターフェース
//!
//! サーバーのシステムトレイアイコンと関連メニューを管理します。

use crate::ui::{ServerState, StatusInfo};
use super::settings::ServerSettings;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(feature = "system-tray")]
use tray_item::{TrayItem, IconSource};

/// システムトレイハンドラ
pub struct TrayHandler {
    /// サーバー状態
    server_state: Arc<Mutex<ServerState>>,
    /// トレイアイテム
    #[cfg(feature = "system-tray")]
    tray: Option<TrayItem>,
    /// トレイワーカースレッド
    worker_thread: Option<thread::JoinHandle<()>>,
    /// 通知を有効化するフラグ
    enable_notifications: bool,
    /// 終了シグナル送信チャネル
    exit_sender: Option<std::sync::mpsc::Sender<()>>,
    /// 設定変更コールバック
    settings_callback: Option<Box<dyn Fn() + Send>>,
}

impl TrayHandler {
    /// 新しいシステムトレイハンドラを作成
    pub fn new(server_state: Arc<Mutex<ServerState>>) -> Self {
        Self {
            server_state,
            #[cfg(feature = "system-tray")]
            tray: None,
            worker_thread: None,
            enable_notifications: true,
            exit_sender: None,
            settings_callback: None,
        }
    }
    
    /// トレイアイコンを初期化
    pub fn initialize(&mut self) -> Result<(), String> {
        #[cfg(feature = "system-tray")]
        {
            // トレイアイコンを作成
            let mut tray = TrayItem::new("リモートデスクトップサーバー", IconSource::Resource("tray-icon"))
                .map_err(|e| format!("トレイアイコンの作成に失敗しました: {}", e))?;
            
            // 終了用チャネルを作成
            let (tx, rx) = std::sync::mpsc::channel();
            self.exit_sender = Some(tx);
            
            // サーバー状態を取得
            let server_state = self.server_state.clone();
            
            // メニューアイテムを追加: サーバー開始/停止
            let tx_start_stop = self.exit_sender.as_ref().unwrap().clone();
            let server_state_start_stop = server_state.clone();
            tray.add_menu_item("サーバーを停止", move || {
                let mut state = server_state_start_stop.lock().unwrap();
                state.running = !state.running;
                
                // サーバー状態変更通知用に専用のチャネルが必要
                // この実装では単純化のため、実際のサーバー制御は別の場所で行います
            })
            .map_err(|e| format!("メニュー項目の追加に失敗しました: {}", e))?;
            
            // メニューアイテムを追加: 設定
            let tx_settings = self.exit_sender.as_ref().unwrap().clone();
            let settings_callback = self.settings_callback.clone();
            tray.add_menu_item("設定...", move || {
                if let Some(callback) = &settings_callback {
                    callback();
                }
            })
            .map_err(|e| format!("メニュー項目の追加に失敗しました: {}", e))?;
            
            // メニューアイテムを追加: 接続クライアント表示
            let tx_clients = self.exit_sender.as_ref().unwrap().clone();
            let server_state_clients = server_state.clone();
            tray.add_menu_item("接続クライアント", move || {
                let state = server_state_clients.lock().unwrap();
                let clients_info = state.sessions.iter()
                    .map(|s| format!("{} ({})", s.ip_address, if s.authenticated { "認証済" } else { "未認証" }))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                let msg = if state.sessions.is_empty() {
                    "接続クライアントはありません".to_string()
                } else {
                    format!("接続クライアント:\n{}", clients_info)
                };
                
                // 実際のアプリではOS固有の通知APIを使用
                println!("{}", msg);
            })
            .map_err(|e| format!("メニュー項目の追加に失敗しました: {}", e))?;
            
            // セパレータを追加
            tray.add_menu_item("-", || {})
                .map_err(|e| format!("セパレータの追加に失敗しました: {}", e))?;
            
            // メニューアイテムを追加: 終了
            let tx_exit = self.exit_sender.as_ref().unwrap().clone();
            tray.add_menu_item("終了", move || {
                let _ = tx_exit.send(());
            })
            .map_err(|e| format!("メニュー項目の追加に失敗しました: {}", e))?;
            
            self.tray = Some(tray);
            
            // 状態監視スレッドを起動
            let server_state_worker = server_state.clone();
            let worker_thread = thread::spawn(move || {
                let mut last_connected_clients = 0;
                
                loop {
                    // 終了要求チェック
                    if let Ok(_) = rx.try_recv() {
                        break;
                    }
                    
                    // サーバー状態を確認
                    let state = server_state_worker.lock().unwrap();
                    
                    // 接続クライアント数の変化を監視
                    if state.connected_clients != last_connected_clients {
                        // 実際のアプリではOS固有の通知APIを使用
                        if state.connected_clients > last_connected_clients {
                            println!("新しいクライアントが接続しました: 合計 {} 接続", state.connected_clients);
                        } else {
                            println!("クライアントが切断しました: 合計 {} 接続", state.connected_clients);
                        }
                        
                        last_connected_clients = state.connected_clients;
                    }
                    
                    // スレッドスリープ
                    drop(state);
                    thread::sleep(Duration::from_secs(1));
                }
            });
            
            self.worker_thread = Some(worker_thread);
            
            Ok(())
        }
        
        #[cfg(not(feature = "system-tray"))]
        {
            Err("システムトレイ機能はサポートされていません".to_string())
        }
    }
    
    /// 状態を更新
    pub fn update_status(&mut self, running: bool, clients: usize) {
        let mut state = self.server_state.lock().unwrap();
        state.running = running;
        state.connected_clients = clients;
    }
    
    /// 通知を送信
    pub fn send_notification(&self, title: &str, message: &str) {
        if !self.enable_notifications {
            return;
        }
        
        #[cfg(feature = "system-tray")]
        {
            // 実際のアプリではOS固有の通知APIを使用
            println!("{}: {}", title, message);
        }
    }
    
    /// トレイアイコンのツールチップを更新
    pub fn update_tooltip(&mut self, tooltip: &str) {
        #[cfg(feature = "system-tray")]
        {
            if let Some(tray) = &mut self.tray {
                let _ = tray.set_tooltip(tooltip);
            }
        }
    }
    
    /// 設定変更コールバックを設定
    pub fn set_settings_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.settings_callback = Some(Box::new(callback));
    }
    
    /// 終了
    pub fn exit(&mut self) {
        // 終了シグナルを送信
        if let Some(tx) = &self.exit_sender {
            let _ = tx.send(());
        }
        
        // ワーカースレッドの終了を待機
        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }
        
        // トレイアイコンをクリーンアップ
        #[cfg(feature = "system-tray")]
        {
            self.tray = None;
        }
    }
}

impl Drop for TrayHandler {
    fn drop(&mut self) {
        self.exit();
    }
}