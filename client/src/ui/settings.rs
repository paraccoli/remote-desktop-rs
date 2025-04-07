//! 設定パネル
//!
//! アプリケーション設定のUIを実装します。

use egui::{Ui, Grid, ComboBox};

/// アプリケーション設定
#[derive(Debug, Clone)]
pub struct AppSettings {
    /// ネットワーク設定
    pub network: NetworkSettings,
    /// 表示設定
    pub display: DisplaySettings,
    /// 入力設定
    pub input: InputSettings,
    /// セキュリティ設定
    pub security: SecuritySettings,
}

/// ネットワーク設定
#[derive(Debug, Clone)]
pub struct NetworkSettings {
    /// デフォルトのホスト
    pub default_host: String,
    /// デフォルトのポート
    pub default_port: u16,
    /// 接続タイムアウト (ミリ秒)
    pub timeout_ms: u64,
    /// 自動再接続
    pub auto_reconnect: bool,
    /// 接続履歴を保存
    pub save_history: bool,
    /// 優先プロトコル
    pub preferred_protocol: String,
}

/// 表示設定
#[derive(Debug, Clone)]
pub struct DisplaySettings {
    /// デフォルトの画質
    pub default_quality: u8,
    /// 色深度
    pub color_depth: u8,
    /// スケーリング方法
    pub scaling_method: String,
    /// アスペクト比を保持
    pub keep_aspect_ratio: bool,
    /// フルスクリーン時にコントロールを自動的に隠す
    pub auto_hide_controls: bool,
    /// デフォルトの更新間隔 (ミリ秒)
    pub default_update_interval: u64,
}

/// 入力設定
#[derive(Debug, Clone)]
pub struct InputSettings {
    /// キーボード入力を送信
    pub send_keyboard: bool,
    /// マウス入力を送信
    pub send_mouse: bool,
    /// 相対マウス移動を使用
    pub use_relative_mouse: bool,
    /// マウスホイールスピード
    pub scroll_speed: f32,
    /// 修飾キーをスティッキーに
    pub sticky_modifiers: bool,
    /// キーマッピングを使用
    pub use_key_mapping: bool,
}

/// セキュリティ設定
#[derive(Debug, Clone)]
pub struct SecuritySettings {
    /// 暗号化を使用
    pub use_encryption: bool,
    /// 認証情報を保存
    pub save_credentials: bool,
    /// 接続時に確認
    pub confirm_on_connect: bool,
    /// 読み取り専用モードをデフォルトに
    pub default_view_only: bool,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            default_host: "localhost".to_string(),
            default_port: 9999,
            timeout_ms: 5000,
            auto_reconnect: false,
            save_history: true,
            preferred_protocol: "tcp".to_string(),
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            default_quality: 50,
            color_depth: 32,
            scaling_method: "bilinear".to_string(),
            keep_aspect_ratio: true,
            auto_hide_controls: true,
            default_update_interval: 200,
        }
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            send_keyboard: true,
            send_mouse: true,
            use_relative_mouse: false,
            scroll_speed: 1.0,
            sticky_modifiers: false,
            use_key_mapping: false,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            use_encryption: true,
            save_credentials: false,
            confirm_on_connect: true,
            default_view_only: false,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            network: NetworkSettings::default(),
            display: DisplaySettings::default(),
            input: InputSettings::default(),
            security: SecuritySettings::default(),
        }
    }
}

/// 設定パネル
pub struct SettingsPanel {
    /// 現在のタブ
    current_tab: String,
}

impl SettingsPanel {
    /// 新しい設定パネルを作成
    pub fn new() -> Self {
        Self {
            current_tab: "ネットワーク".to_string(),
        }
    }
    
    /// UIに表示
    pub fn ui(&mut self, ui: &mut Ui, settings: &mut AppSettings) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tab, "ネットワーク".to_string(), "ネットワーク");
            ui.selectable_value(&mut self.current_tab, "表示".to_string(), "表示");
            ui.selectable_value(&mut self.current_tab, "入力".to_string(), "入力");
            ui.selectable_value(&mut self.current_tab, "セキュリティ".to_string(), "セキュリティ");
        });
        
        ui.separator();
        
        match self.current_tab.as_str() {
            "ネットワーク" => self.show_network_settings(ui, &mut settings.network),
            "表示" => self.show_display_settings(ui, &mut settings.display),
            "入力" => self.show_input_settings(ui, &mut settings.input),
            "セキュリティ" => self.show_security_settings(ui, &mut settings.security),
            _ => {}
        }
    }
    
    /// ネットワーク設定を表示
    fn show_network_settings(&self, ui: &mut Ui, settings: &mut NetworkSettings) {
        Grid::new("network_settings_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("デフォルトホスト:");
                ui.text_edit_singleline(&mut settings.default_host);
                ui.end_row();
                
                ui.label("デフォルトポート:");
                ui.add(egui::DragValue::new(&mut settings.default_port).speed(1.0));
                ui.end_row();
                
                ui.label("接続タイムアウト (ms):");
                ui.add(egui::DragValue::new(&mut settings.timeout_ms).speed(100.0));
                ui.end_row();
                
                ui.label("優先プロトコル:");
                ComboBox::from_id_source("preferred_protocol")
                    .selected_text(&settings.preferred_protocol)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.preferred_protocol, "tcp".to_string(), "TCP");
                        ui.selectable_value(&mut settings.preferred_protocol, "websocket".to_string(), "WebSocket");
                        ui.selectable_value(&mut settings.preferred_protocol, "webrtc".to_string(), "WebRTC");
                    });
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.auto_reconnect, "自動再接続");
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.save_history, "接続履歴を保存");
                ui.end_row();
            });
    }
    
    /// 表示設定を表示
    fn show_display_settings(&self, ui: &mut Ui, settings: &mut DisplaySettings) {
        Grid::new("display_settings_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("デフォルト画質:");
                ui.add(egui::Slider::new(&mut settings.default_quality, 10..=90));
                ui.end_row();
                
                ui.label("色深度:");
                ComboBox::from_id_source("color_depth")
                    .selected_text(format!("{}ビット", settings.color_depth))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.color_depth, 8, "8ビット");
                        ui.selectable_value(&mut settings.color_depth, 16, "16ビット");
                        ui.selectable_value(&mut settings.color_depth, 24, "24ビット");
                        ui.selectable_value(&mut settings.color_depth, 32, "32ビット");
                    });
                ui.end_row();
                
                ui.label("スケーリング方法:");
                ComboBox::from_id_source("scaling_method")
                    .selected_text(&settings.scaling_method)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.scaling_method, "nearest".to_string(), "ニアレストネイバー");
                        ui.selectable_value(&mut settings.scaling_method, "bilinear".to_string(), "バイリニア");
                        ui.selectable_value(&mut settings.scaling_method, "bicubic".to_string(), "バイキュービック");
                    });
                ui.end_row();
                
                ui.label("更新間隔 (ms):");
                ui.add(egui::DragValue::new(&mut settings.default_update_interval).speed(10.0).clamp_range(50..=1000));
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.keep_aspect_ratio, "アスペクト比を保持");
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.auto_hide_controls, "フルスクリーン時にコントロールを自動的に隠す");
                ui.end_row();
            });
    }
    
    /// 入力設定を表示
    fn show_input_settings(&self, ui: &mut Ui, settings: &mut InputSettings) {
        Grid::new("input_settings_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("");
                ui.checkbox(&mut settings.send_keyboard, "キーボード入力を送信");
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.send_mouse, "マウス入力を送信");
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.use_relative_mouse, "相対マウス移動を使用");
                ui.end_row();
                
                ui.label("スクロール速度:");
                ui.add(egui::Slider::new(&mut settings.scroll_speed, 0.1..=3.0));
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.sticky_modifiers, "修飾キーをスティッキーに");
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.use_key_mapping, "キーマッピングを使用");
                ui.end_row();
                
                if settings.use_key_mapping {
                    ui.label("");
                    if ui.button("キーマッピング設定...").clicked() {
                        // TODO: キーマッピング設定ダイアログを表示
                    }
                    ui.end_row();
                }
            });
    }
    
    /// セキュリティ設定を表示
    fn show_security_settings(&self, ui: &mut Ui, settings: &mut SecuritySettings) {
        Grid::new("security_settings_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("");
                ui.checkbox(&mut settings.use_encryption, "暗号化を使用");
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.save_credentials, "認証情報を保存");
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.confirm_on_connect, "接続時に確認");
                ui.end_row();
                
                ui.label("");
                ui.checkbox(&mut settings.default_view_only, "読み取り専用モードをデフォルトに");
                ui.end_row();
            });
    }
}