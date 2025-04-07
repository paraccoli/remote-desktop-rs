//! メインアプリケーション
//!
//! リモートデスクトップクライアントのメインアプリケーションを実装します。

use crate::ui::{MainWindow, AppState, AppSettings};
use crate::network::{ConnectionInfo, TcpClient, WebSocketClient, WebRtcClient, NetworkClient};
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};

/// アプリケーション
pub struct App {
    /// 設定
    settings: AppSettings,
    /// 設定ファイルのパス
    settings_path: PathBuf,
    /// 接続履歴
    connection_history: Vec<ConnectionInfo>,
    /// 最後の接続情報
    last_connection: Option<ConnectionInfo>,
}

/// 保存される設定
#[derive(Serialize, Deserialize, Clone, Debug)]
struct SavedSettings {
    /// アプリケーション設定
    app_settings: AppSettings,
    /// 接続履歴
    connection_history: Vec<ConnectionInfo>,
    /// 最後の接続情報
    last_connection: Option<ConnectionInfo>,
}

impl App {
    /// 新しいアプリケーションを作成
    pub fn new() -> Self {
        // 設定ファイルのパスを取得
        let settings_path = Self::get_settings_path();
        
        // 設定を読み込み
        let (settings, connection_history, last_connection) = Self::load_settings(&settings_path);
        
        Self {
            settings,
            settings_path,
            connection_history,
            last_connection,
        }
    }
    
    /// 設定ファイルのパスを取得
    fn get_settings_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        
        let app_config_dir = config_dir.join("remote-desktop-rs");
        
        // ディレクトリが存在しない場合は作成
        if !app_config_dir.exists() {
            if let Err(e) = fs::create_dir_all(&app_config_dir) {
                eprintln!("設定ディレクトリの作成に失敗しました: {}", e);
            }
        }
        
        app_config_dir.join("settings.json")
    }
    
    /// 設定を読み込む
    fn load_settings(path: &Path) -> (AppSettings, Vec<ConnectionInfo>, Option<ConnectionInfo>) {
        // ファイルが存在しない場合はデフォルト値を返す
        if !path.exists() {
            return (AppSettings::default(), Vec::new(), None);
        }
        
        // ファイルを読み込み
        match fs::read_to_string(path) {
            Ok(content) => {
                // JSONをパース
                match serde_json::from_str::<SavedSettings>(&content) {
                    Ok(saved) => {
                        (saved.app_settings, saved.connection_history, saved.last_connection)
                    },
                    Err(e) => {
                        eprintln!("設定ファイルのパースに失敗しました: {}", e);
                        (AppSettings::default(), Vec::new(), None)
                    }
                }
            },
            Err(e) => {
                eprintln!("設定ファイルの読み込みに失敗しました: {}", e);
                (AppSettings::default(), Vec::new(), None)
            }
        }
    }
    
    /// 設定を保存
    pub fn save_settings(&self) {
        let saved = SavedSettings {
            app_settings: self.settings.clone(),
            connection_history: self.connection_history.clone(),
            last_connection: self.last_connection.clone(),
        };
        
        // JSONに変換
        match serde_json::to_string_pretty(&saved) {
            Ok(json) => {
                // ファイルに保存
                if let Err(e) = fs::write(&self.settings_path, json) {
                    eprintln!("設定ファイルの保存に失敗しました: {}", e);
                }
            },
            Err(e) => {
                eprintln!("設定のシリアライズに失敗しました: {}", e);
            }
        }
    }
    
    /// 接続履歴に追加
    pub fn add_to_history(&mut self, info: ConnectionInfo) {
        // 既に同じエントリがある場合は削除
        self.connection_history.retain(|i| i.host != info.host || i.port != info.port);
        
        // 先頭に追加
        self.connection_history.insert(0, info.clone());
        
        // 履歴の最大数を制限
        if self.connection_history.len() > 10 {
            self.connection_history.truncate(10);
        }
        
        // 最後の接続情報を更新
        self.last_connection = Some(info);
        
        // 設定を保存
        self.save_settings();
    }
    
    /// アプリケーション設定を取得
    pub fn get_settings(&self) -> &AppSettings {
        &self.settings
    }
    
    /// アプリケーション設定を更新
    pub fn update_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
        self.save_settings();
    }
    
    /// 接続履歴を取得
    pub fn get_connection_history(&self) -> &[ConnectionInfo] {
        &self.connection_history
    }
    
    /// 最後の接続情報を取得
    pub fn get_last_connection(&self) -> Option<&ConnectionInfo> {
        self.last_connection.as_ref()
    }
    
    /// 接続クライアントを作成
    pub fn create_client(&self, info: &ConnectionInfo) -> Result<Box<dyn NetworkClient>, String> {
        match info.protocol.as_str() {
            "tcp" => Ok(Box::new(TcpClient::new())),
            "websocket" => Ok(Box::new(WebSocketClient::new())),
            "webrtc" => {
                match WebRtcClient::new() {
                    Ok(client) => Ok(Box::new(client)),
                    Err(e) => Err(format!("WebRTCクライアントの初期化に失敗しました: {}", e)),
                }
            },
            _ => Err("不明な接続プロトコルです".to_string()),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}