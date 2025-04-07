//! UIモジュール
//!
//! このモジュールはリモートデスクトップクライアントのユーザーインターフェースを担当します。

mod window;
mod controls;
mod settings;
mod styles;

pub use window::MainWindow;
pub use controls::{ControlPanel, ToolButton};
pub use settings::{SettingsPanel, AppSettings};
pub use styles::{Styles, Theme, ColorScheme};

/// アプリケーションの状態
#[derive(Debug, Clone)]
pub struct AppState {
    /// 接続状態
    pub connected: bool,
    /// 現在の設定
    pub settings: AppSettings,
    /// 表示モード
    pub display_mode: DisplayMode,
    /// パフォーマンス情報
    pub performance: PerformanceInfo,
}

/// 表示モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// ウィンドウモード
    Window,
    /// フルスクリーンモード
    Fullscreen,
    /// スケーリングモード (アスペクト比を維持)
    Scaled,
    /// 1:1 表示モード
    OneToOne,
}

/// パフォーマンス情報
#[derive(Debug, Clone)]
pub struct PerformanceInfo {
    /// FPS (フレーム毎秒)
    pub fps: f32,
    /// レイテンシー (ミリ秒)
    pub latency: u64,
    /// 現在の画質
    pub quality: u8,
    /// フレーム処理時間 (ミリ秒)
    pub frame_time: f32,
    /// 転送データサイズ (バイト/秒)
    pub data_rate: u64,
}

impl Default for PerformanceInfo {
    fn default() -> Self {
        Self {
            fps: 0.0,
            latency: 0,
            quality: 50,
            frame_time: 0.0,
            data_rate: 0,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connected: false,
            settings: AppSettings::default(),
            display_mode: DisplayMode::Scaled,
            performance: PerformanceInfo::default(),
        }
    }
}