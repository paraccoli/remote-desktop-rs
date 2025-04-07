//! クライアントエントリポイント
//!
//! リモートデスクトップクライアントのメインエントリポイント

use eframe::egui;
use remote_desktop_rs_client::ui::MainWindow;

fn main() {
    // ネイティブオプションを設定
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        min_window_size: Some(egui::vec2(640.0, 480.0)),
        resizable: true,
        maximized: false,
        decorated: true,
        transparent: false,
        vsync: true,
        icon_data: load_icon(),
        ..Default::default()
    };

    // アプリケーションを実行
    eframe::run_native(
        "リモートデスクトップクライアント",
        native_options,
        Box::new(|cc| Box::new(MainWindow::new(cc))),
    ).expect("アプリケーションの起動に失敗しました");
}

/// アプリケーションアイコンを読み込む
fn load_icon() -> Option<eframe::IconData> {
    // アイコンファイルのパス
    let icon_path = std::path::Path::new("assets/app.png");
    
    if icon_path.exists() {
        // 画像を読み込み
        let image = image::open(icon_path).ok()?;
        let image = image.to_rgba8();
        
        // アイコンデータを作成
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        
        Some(eframe::IconData {
            rgba,
            width,
            height,
        })
    } else {
        println!("警告: アイコンファイルが見つかりませんでした: {:?}", icon_path);
        None
    }
}