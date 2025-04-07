//! Webクライアントエントリポイント
//!
//! このクレートは、リモートデスクトップのWebクライアント実装を提供します。
//! WebAssemblyにコンパイルされ、ブラウザ上で実行されます。

mod app;
mod components;
mod state;
mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, Element, HtmlElement};

/// Webクライアントの初期化
#[wasm_bindgen]
pub fn initialize() -> Result<(), JsValue> {
    // パニック時のフックを設定
    #[cfg(feature = "development")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    // ロガーを初期化
    wasm_logger::init(wasm_logger::Config::default());
    
    log::info!("Webクライアントを初期化中...");
    
    // DOMが読み込まれているか確認
    let window = window().ok_or_else(|| JsValue::from_str("ウィンドウが見つかりません"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("ドキュメントが見つかりません"))?;
    
    // アプリケーションのコンテナを取得
    if let Some(app_container) = document.get_element_by_id("app") {
        // Yewアプリケーションをマウント
        yew::Renderer::<app::App>::with_root(app_container).render();
        log::info!("アプリケーションを正常にマウントしました");
    } else {
        // コンテナが存在しない場合は作成してマウント
        log::warn!("アプリケーションコンテナが見つかりません。新しく作成します。");
        
        let body = document.body().ok_or_else(|| JsValue::from_str("ドキュメントのボディが見つかりません"))?;
        
        let app_div = document.create_element("div")?;
        app_div.set_id("app");
        app_div.set_class_name("remote-desktop-app");
        
        body.append_child(&app_div)?;
        
        // Yewアプリケーションをマウント
        yew::Renderer::<app::App>::with_root(app_div).render();
        log::info!("アプリケーションを新しく作成したコンテナにマウントしました");
    }
    
    log::info!("Webクライアントの初期化が完了しました");
    Ok(())
}

/// Webクライアントの状態をチェック
#[wasm_bindgen]
pub fn check_status() -> bool {
    log::info!("Webクライアントのステータスをチェック中");
    true
}

/// バージョン情報を取得
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Webクライアントをシャットダウン
#[wasm_bindgen]
pub fn shutdown() -> Result<(), JsValue> {
    log::info!("Webクライアントをシャットダウン中...");
    
    // シャットダウン処理が必要な場合はここに実装
    
    Ok(())
}

/// アプリケーションの状態
#[derive(Debug, Clone)]
pub struct AppState {
    /// 接続状態
    pub connected: bool,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// 画面データ
    pub image_data: Option<Vec<u8>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connected: false,
            error_message: None,
            image_data: None,
        }
    }
}