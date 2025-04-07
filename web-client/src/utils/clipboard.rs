//! クリップボードユーティリティ
//!
//! ブラウザのクリップボードAPIを使用してテキストのコピー・貼り付けを行うユーティリティ関数を提供します。

use wasm_bindgen::prelude::*;
use web_sys::{window, Navigator, Clipboard, Window};
use wasm_bindgen_futures::JsFuture;
use js_sys::Promise;
use wasm_bindgen::JsCast;

pub enum Error {
    ApiNotSupported,
    CopyFailed,
    PasteFailed,
}

/// クリップボードにテキストをコピー
pub async fn copy_to_clipboard(text: &str) -> Result<(), JsValue> {
    if let Some(clipboard) = get_clipboard() {
        // write_text()はPromiseを返すのでJsFutureに変換
        JsFuture::from(clipboard.write_text(text)).await?;
        Ok(())
    } else {
        Err(JsValue::from_str("クリップボードAPIが利用できません"))
    }
}

/// クリップボードからテキストを取得
pub async fn read_from_clipboard() -> Result<String, JsValue> {
    if let Some(clipboard) = get_clipboard() {
        // read_text()はPromiseを返すのでJsFutureに変換
        let text = JsFuture::from(clipboard.read_text()).await?;
        
        // JsValueをStringに変換
        match text.as_string() {
            Some(s) => Ok(s),
            None => Err(JsValue::from_str("テキストの変換に失敗しました")),
        }
    } else {
        Err(JsValue::from_str("クリップボードAPIが利用できません"))
    }
}

/// クリップボードAPIが利用可能かチェック
pub fn is_clipboard_available() -> bool {
    get_clipboard().is_some()
}

/// クリップボードオブジェクトを取得
fn get_clipboard() -> Option<Clipboard> {
    window()
        .and_then(|win| {
            win.navigator().dyn_into::<web_sys::Navigator>().ok()
                .and_then(|nav| nav.clipboard().ok())
        })
}

/// レガシー方式でクリップボードにコピー（document.execCommand）
/// 
/// 注: この方法は非推奨です。可能であればcopy_to_clipboard()を使用してください。
pub fn legacy_copy_to_clipboard(text: &str) -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("windowが取得できません"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("documentが取得できません"))?;
    
    // テキストエリア要素を作成
    let textarea = document
        .create_element("textarea")?
        .dyn_into::<web_sys::HtmlTextAreaElement>()?;
    
    // テキストをセット
    textarea.set_value(text);
    
    // DOMに追加
    let body = document.body().ok_or_else(|| JsValue::from_str("bodyが取得できません"))?;
    body.append_child(&textarea)?;
    
    // テキストを選択
    textarea.select();
    
    // コピーコマンドを実行
    let range = document.create_range().map_err(|_| Error::ApiNotSupported)?;
    let selection = window().and_then(|win| win.get_selection().ok()).flatten()
        .ok_or(Error::ApiNotSupported)?;
    // 選択範囲をクリアして新しく選択
    selection.remove_all_ranges().map_err(|_| Error::ApiNotSupported)?;
    selection.add_range(&range).map_err(|_| Error::ApiNotSupported)?;
    // コピーをシミュレート (これは理想的な方法ではありませんが、代替手段として)
    
    // DOMから削除
    body.remove_child(&textarea)?;
    
    if result {
        Ok(())
    } else {
        Err(JsValue::from_str("コピーに失敗しました"))
    }
}

/// クリップボード操作に関する権限を要求
pub async fn request_clipboard_permission() -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("windowが取得できません"))?;
    let navigator = window.navigator();
    
    // Permissions APIが利用可能か確認
    if js_sys::Reflect::has(&navigator, &JsValue::from_str("permissions"))? {
        let permissions = js_sys::Reflect::get(&navigator, &JsValue::from_str("permissions"))?;
        
        // クリップボード権限を要求
        let clipboard_permission = js_sys::Object::new();
        js_sys::Reflect::set(
            &clipboard_permission,
            &JsValue::from_str("name"),
            &JsValue::from_str("clipboard-read")
        )?;
        
        let query_result = js_sys::Reflect::get(&permissions, &JsValue::from_str("query"))?;
        let query_fn = query_result.dyn_into::<js_sys::Function>()?;
        
        let promise = query_fn.call1(&permissions, &clipboard_permission)?
            .dyn_into::<Promise>()?;
        
        // 権限結果を待機
        let _permission_status = JsFuture::from(promise).await?;
        
        // 書き込み権限も要求
        let write_permission = js_sys::Object::new();
        js_sys::Reflect::set(
            &write_permission,
            &JsValue::from_str("name"),
            &JsValue::from_str("clipboard-write")
        )?;
        
        let promise = query_fn.call1(&permissions, &write_permission)?
            .dyn_into::<Promise>()?;
        
        let _permission_status = JsFuture::from(promise).await?;
        
        Ok(())
    } else {
        // Permissions APIが利用できない場合は、シンプルに試してみる
        read_from_clipboard().await?;
        Ok(())
    }
}