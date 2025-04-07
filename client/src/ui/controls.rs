//! 操作コントロール
//!
//! リモートサーバーへの操作コントロールパネルを実装します。

use super::{AppState, DisplayMode};
use egui::{Ui, Vec2, RichText, Color32};

/// ツールボタン
pub struct ToolButton {
    /// ボタン名
    name: String,
    /// アイコン名（FontAwesomeなど）
    icon: Option<String>,
    /// ツールチップ
    tooltip: String,
    /// 有効状態
    enabled: bool,
    /// トグル状態
    toggled: bool,
}

impl ToolButton {
    /// 新しいツールボタンを作成
    pub fn new(name: impl Into<String>, tooltip: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            icon: None,
            tooltip: tooltip.into(),
            enabled: true,
            toggled: false,
        }
    }
    
    /// アイコンを設定
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    /// 有効状態を設定
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// トグル状態を設定
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }
    
    /// UIに表示
    pub fn ui(&mut self, ui: &mut Ui) -> bool {
        let text = if let Some(icon) = &self.icon {
            format!("{} {}", icon, self.name)
        } else {
            self.name.clone()
        };
        
        let text = if self.toggled {
            RichText::new(text).color(Color32::LIGHT_BLUE)
        } else {
            RichText::new(text)
        };
        
        let button = egui::Button::new(text).enabled(self.enabled);
        
        let response = ui.add(button);
        
        if !self.tooltip.is_empty() {
            response.on_hover_text(&self.tooltip);
        }
        
        response.clicked()
    }
}

/// コントロールパネル
pub struct ControlPanel {
    /// ツールボタン群
    tools: Vec<ToolButton>,
    /// アプリケーション実行入力
    run_app_input: String,
    /// 特殊キー状態
    special_keys: bool,
}

impl ControlPanel {
    /// 新しいコントロールパネルを作成
    pub fn new() -> Self {
        Self {
            tools: vec![
                ToolButton::new("全画面", "フルスクリーンモードに切替").with_icon("⛶"),
                ToolButton::new("実寸表示", "1:1サイズで表示").with_icon("🔍"),
                ToolButton::new("画面キャプチャ", "スクリーンショットを保存").with_icon("📷"),
            ],
            run_app_input: String::new(),
            special_keys: false,
        }
    }
    
    /// UIに表示
    pub fn ui(&mut self, ui: &mut Ui, app_state: &mut AppState) {
        ui.heading("コントロール");
        
        ui.add_space(8.0);
        
        // 表示モードコントロール
        ui.label("表示モード");
        ui.horizontal(|ui| {
            if ui.selectable_label(app_state.display_mode == DisplayMode::Window, "ウィンドウ").clicked() {
                app_state.display_mode = DisplayMode::Window;
            }
            if ui.selectable_label(app_state.display_mode == DisplayMode::Fullscreen, "全画面").clicked() {
                app_state.display_mode = DisplayMode::Fullscreen;
            }
            if ui.selectable_label(app_state.display_mode == DisplayMode::OneToOne, "実寸").clicked() {
                app_state.display_mode = DisplayMode::OneToOne;
            }
        });
        
        ui.add_space(8.0);
        
        // ツールボタン群
        ui.label("ツール");
        ui.horizontal_wrapped(|ui| {
            for tool in &mut self.tools {
                let button_size = Vec2::new(80.0, 32.0);
                ui.set_min_size(button_size);
                if tool.ui(ui) {
                    match tool.name.as_str() {
                        "全画面" => {
                            app_state.display_mode = DisplayMode::Fullscreen;
                        },
                        "実寸表示" => {
                            app_state.display_mode = DisplayMode::OneToOne;
                        },
                        "画面キャプチャ" => {
                            // TODO: スクリーンショット保存
                        },
                        _ => {}
                    }
                }
            }
        });
        
        ui.add_space(8.0);
        
        // アプリケーション実行
        ui.label("アプリケーション実行");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.run_app_input);
            if ui.button("実行").clicked() && !self.run_app_input.is_empty() {
                // TODO: アプリケーション実行コマンド
                self.run_app_input.clear();
            }
        });
        
        ui.add_space(8.0);
        
        // 特殊キー操作
        ui.label("特殊キー操作");
        ui.horizontal(|ui| {
            if ui.button("Ctrl+Alt+Del").clicked() {
                // TODO: Ctrl+Alt+Del送信
            }
            if ui.button("Win+D").clicked() {
                // TODO: Win+D送信
            }
            if ui.button("Alt+Tab").clicked() {
                // TODO: Alt+Tab送信
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("Esc").clicked() {
                // TODO: Esc送信
            }
            if ui.button("PrintScreen").clicked() {
                // TODO: PrintScreen送信
            }
            if ui.button("Win").clicked() {
                // TODO: Win送信
            }
        });
        
        ui.add_space(8.0);
        
        // ファンクションキー
        ui.label("ファンクションキー");
        ui.horizontal_wrapped(|ui| {
            for i in 1..=12 {
                if ui.button(format!("F{}", i)).clicked() {
                    // TODO: ファンクションキー送信
                }
            }
        });
        
        ui.add_space(8.0);
        
        // パフォーマンス情報
        ui.separator();
        ui.label("パフォーマンス");
        ui.label(format!("FPS: {:.1}", app_state.performance.fps));
        ui.label(format!("レイテンシー: {}ms", app_state.performance.latency));
        ui.label(format!("フレーム時間: {:.1}ms", app_state.performance.frame_time));
    }
}