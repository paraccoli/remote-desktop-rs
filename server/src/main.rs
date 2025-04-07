//! リモートデスクトップサーバーエントリポイント
//!
//! リモートデスクトップサーバーのメインエントリポイント

mod app;
mod capture;
mod input;
mod network;
mod ui;

use app::App;
use std::process;

fn main() {
    // ロガーを初期化
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    // パニックハンドラを設定（未処理のパニックをログに記録）
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("サーバーでパニックが発生しました: {:?}", panic_info);
        log::error!("サーバーでパニックが発生しました: {:?}", panic_info);
    }));
    
    // コマンドライン引数を解析
    let args: Vec<String> = std::env::args().collect();
    
    // ヘルプ表示
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        println!("リモートデスクトップサーバー v{}", env!("CARGO_PKG_VERSION"));
        println!("使用方法: remote-desktop-server [オプション]");
        println!("オプション:");
        println!("  --no-gui       GUIを表示せずに起動");
        println!("  --autostart    自動的にサーバーを開始");
        println!("  --config=FILE  指定した設定ファイルを使用");
        println!("  --port=PORT    指定したポートで起動");
        println!("  --help, -h     このヘルプを表示");
        return;
    }
    
    // バージョン表示
    if args.len() > 1 && (args[1] == "-v" || args[1] == "--version") {
        println!("リモートデスクトップサーバー v{}", env!("CARGO_PKG_VERSION"));
        return;
    }
    
    // アプリケーションを初期化
    let mut app = match App::new() {
        Ok(app) => app,
        Err(e) => {
            eprintln!("アプリケーションの初期化に失敗しました: {}", e);
            log::error!("アプリケーションの初期化に失敗しました: {}", e);
            process::exit(1);
        }
    };
    
    // アプリケーションを実行
    if let Err(e) = app.run() {
        eprintln!("アプリケーションの実行中にエラーが発生しました: {}", e);
        log::error!("アプリケーションの実行中にエラーが発生しました: {}", e);
        process::exit(1);
    }
}