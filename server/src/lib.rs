//! リモートデスクトップサーバーライブラリ
//!
//! このクレートはリモートデスクトップサーバーの機能を提供します。

// 必要なモジュールのエクスポート
pub mod app;
pub mod config;
pub mod network;
pub mod capture;
pub mod input;
pub mod ui;
pub mod error;
// 他に必要なモジュール

// main.rsで使用される関数をエクスポート
pub fn run() -> Result<(), String> {
    let mut app = app::App::new()?;
    app.initialize()?;
    app.run()
}