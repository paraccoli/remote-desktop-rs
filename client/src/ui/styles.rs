//! スタイル設定
//!
//! UIスタイルを定義します。

use egui::{FontFamily, FontId, Color32, Stroke, Rounding};

/// カラースキーム
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// 背景色
    pub background: Color32,
    /// 前景色（テキスト）
    pub foreground: Color32,
    /// プライマリカラー
    pub primary: Color32,
    /// セカンダリカラー
    pub secondary: Color32,
    /// アクセントカラー
    pub accent: Color32,
    /// エラーカラー
    pub error: Color32,
    /// 警告カラー
    pub warning: Color32,
    /// 成功カラー
    pub success: Color32,
}

/// テーマ
#[derive(Debug, Clone)]
pub enum Theme {
    /// ライトテーマ
    Light,
    /// ダークテーマ
    Dark,
    /// ハイコントラストテーマ
    HighContrast,
    /// カスタムテーマ
    Custom(ColorScheme),
}

/// スタイル設定
#[derive(Debug, Clone)]
pub struct Styles {
    /// テーマ
    pub theme: Theme,
    /// フォントファミリー
    pub font_family: FontFamily,
    /// 通常テキストのフォントサイズ
    pub font_size: f32,
    /// ボタンの丸み
    pub button_rounding: Rounding,
    /// ウィンドウの丸み
    pub window_rounding: Rounding,
    /// フレーム線の太さ
    pub frame_stroke: Stroke,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: Color32::from_rgb(30, 30, 30),
            foreground: Color32::from_rgb(240, 240, 240),
            primary: Color32::from_rgb(70, 130, 180),
            secondary: Color32::from_rgb(50, 50, 50),
            accent: Color32::from_rgb(100, 150, 200),
            error: Color32::from_rgb(220, 50, 50),
            warning: Color32::from_rgb(220, 180, 50),
            success: Color32::from_rgb(50, 180, 50),
        }
    }
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            font_family: FontFamily::Proportional,
            font_size: 16.0,
            button_rounding: Rounding::same(4.0),
            window_rounding: Rounding::same(6.0),
            frame_stroke: Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
        }
    }
}

impl Styles {
    /// テーマに基づいたカラースキームを取得
    pub fn color_scheme(&self) -> ColorScheme {
        match &self.theme {
            Theme::Light => ColorScheme {
                background: Color32::from_rgb(240, 240, 240),
                foreground: Color32::from_rgb(30, 30, 30),
                primary: Color32::from_rgb(70, 130, 180),
                secondary: Color32::from_rgb(220, 220, 220),
                accent: Color32::from_rgb(100, 150, 200),
                error: Color32::from_rgb(200, 50, 50),
                warning: Color32::from_rgb(200, 150, 50),
                success: Color32::from_rgb(50, 150, 50),
            },
            Theme::Dark => ColorScheme::default(),
            Theme::HighContrast => ColorScheme {
                background: Color32::BLACK,
                foreground: Color32::WHITE,
                primary: Color32::from_rgb(0, 174, 255),
                secondary: Color32::from_rgb(50, 50, 50),
                accent: Color32::from_rgb(255, 215, 0),
                error: Color32::from_rgb(255, 70, 70),
                warning: Color32::from_rgb(255, 215, 0),
                success: Color32::from_rgb(50, 255, 50),
            },
            Theme::Custom(scheme) => scheme.clone(),
        }
    }
    
    /// テキストスタイルを取得
    pub fn text_style(&self) -> FontId {
        FontId::new(self.font_size, self.font_family.clone())
    }
    
    /// 見出しスタイルを取得
    pub fn heading_style(&self) -> FontId {
        FontId::new(self.font_size * 1.5, self.font_family.clone())
    }
    
    /// 小さいテキストスタイルを取得
    pub fn small_text_style(&self) -> FontId {
        FontId::new(self.font_size * 0.8, self.font_family.clone())
    }
    
    /// テーマをEGUIのスタイルに適用
    pub fn apply_to_egui(&self, style: &mut egui::Style) {
        let colors = self.color_scheme();
        
        style.visuals.window_rounding = self.window_rounding;
        style.visuals.window_shadow.extrusion = 8.0;
        
        style.visuals.widgets.noninteractive.rounding = Rounding::same(2.0);
        style.visuals.widgets.inactive.rounding = self.button_rounding;
        style.visuals.widgets.hovered.rounding = self.button_rounding;
        style.visuals.widgets.active.rounding = self.button_rounding;
        
        style.visuals.widgets.noninteractive.bg_fill = colors.secondary;
        style.visuals.widgets.inactive.bg_fill = colors.secondary;
        style.visuals.widgets.hovered.bg_fill = colors.primary;
        style.visuals.widgets.active.bg_fill = colors.accent;
        
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.foreground);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.foreground);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
        style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
        
        style.visuals.selection.bg_fill = colors.accent.linear_multiply(0.5);
        style.visuals.selection.stroke = Stroke::new(1.0, colors.accent);
        
        // 背景色を設定
        style.visuals.extreme_bg_color = colors.background;
        style.visuals.window_fill = colors.background;
        style.visuals.panel_fill = colors.background;
        
        // テキスト色を設定
        style.visuals.override_text_color = Some(colors.foreground);
        
        // ポップアップとメニューのスタイル
        style.visuals.popup_shadow.extrusion = 16.0;
        style.visuals.menu_rounding = Rounding::same(4.0);
        
        // スクロールバーのスタイル
        style.spacing.scroll_bar_width = 8.0;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors.primary.linear_multiply(0.5));
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors.primary);
        
        // テキストカーソルのスタイル
        style.visuals.text_cursor.width = 2.0;
        style.visuals.text_cursor.preview = false;
        
        // ウィンドウの内部余白
        style.spacing.window_padding = egui::vec2(12.0, 12.0);
        
        // アニメーションの持続時間
        style.animation_time = 0.15;
    }
    
    /// ライトテーマを適用
    pub fn light_theme(&mut self) {
        self.theme = Theme::Light;
    }
    
    /// ダークテーマを適用
    pub fn dark_theme(&mut self) {
        self.theme = Theme::Dark;
    }
    
    /// ハイコントラストテーマを適用
    pub fn high_contrast_theme(&mut self) {
        self.theme = Theme::HighContrast;
    }
    
    /// カスタムテーマを適用
    pub fn custom_theme(&mut self, scheme: ColorScheme) {
        self.theme = Theme::Custom(scheme);
    }
    
    /// フォントサイズを変更
    pub fn with_font_size(&mut self, size: f32) {
        self.font_size = size;
    }
    
    /// フォントファミリーを変更
    pub fn with_font_family(&mut self, family: FontFamily) {
        self.font_family = family;
    }
    
    /// ボタンの丸みを設定
    pub fn with_button_rounding(&mut self, rounding: f32) {
        self.button_rounding = Rounding::same(rounding);
    }
    
    /// ウィンドウの丸みを設定
    pub fn with_window_rounding(&mut self, rounding: f32) {
        self.window_rounding = Rounding::same(rounding);
    }
    
    /// フレーム線の太さを設定
    pub fn with_frame_stroke(&mut self, width: f32, color: Color32) {
        self.frame_stroke = Stroke::new(width, color);
    }
}