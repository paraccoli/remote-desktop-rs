//! ストレージユーティリティ
//!
//! ブラウザのローカルストレージを使用して設定の保存と読み込みを行うユーティリティ関数を提供します。

use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use crate::state::AppSettings;

/// ローカルストレージから設定を読み込む
pub fn load_settings<T>() -> Option<T>
where
    T: for<'de> Deserialize<'de> + Default + Debug + 'static,
{
    let storage = get_local_storage()?;
    let settings_key = get_settings_key::<T>();
    
    match storage.get_item(&settings_key) {
        Ok(Some(value)) => {
            match serde_json::from_str::<T>(&value) {
                Ok(settings) => {
                    log::info!("設定を読み込みました: {:?}", settings);
                    Some(settings)
                },
                Err(e) => {
                    log::error!("設定のデシリアライズに失敗しました: {}", e);
                    None
                }
            }
        },
        Ok(None) => {
            log::info!("保存された設定がありません。デフォルト設定を使用します。");
            None
        },
        Err(e) => {
            log::error!("設定の読み込みに失敗しました: {:?}", e);
            None
        }
    }
}

/// ローカルストレージに設定を保存する
pub fn save_settings<T>(settings: &T) -> Result<(), JsValue>
where
    T: Serialize + Debug + 'static,
{
    if let Some(storage) = get_local_storage() {
        let settings_key = get_settings_key::<T>();
        
        match serde_json::to_string(settings) {
            Ok(json) => {
                match storage.set_item(&settings_key, &json) {
                    Ok(_) => {
                        log::info!("設定を保存しました: {:?}", settings);
                        Ok(())
                    },
                    Err(e) => {
                        log::error!("設定の保存に失敗しました: {:?}", e);
                        Err(JsValue::from_str(&format!("設定の保存に失敗しました: {:?}", e)))
                    }
                }
            },
            Err(e) => {
                log::error!("設定のシリアライズに失敗しました: {}", e);
                Err(JsValue::from_str(&format!("設定のシリアライズに失敗しました: {}", e)))
            }
        }
    } else {
        log::error!("ローカルストレージを取得できませんでした");
        Err(JsValue::from_str("ローカルストレージを取得できませんでした"))
    }
}

/// ローカルストレージから項目を削除する
pub fn remove_settings<T>() -> bool 
where 
    T: 'static
{
    if let Some(storage) = get_local_storage() {
        let settings_key = get_settings_key::<T>();
        
        match storage.remove_item(&settings_key) {
            Ok(_) => {
                log::info!("設定を削除しました");
                true
            },
            Err(e) => {
                log::error!("設定の削除に失敗しました: {:?}", e);
                false
            }
        }
    } else {
        log::error!("ローカルストレージを取得できませんでした");
        false
    }
}

/// ローカルストレージをクリアする
pub fn clear_all_settings() -> bool {
    if let Some(storage) = get_local_storage() {
        match storage.clear() {
            Ok(_) => {
                log::info!("全ての設定を削除しました");
                true
            },
            Err(e) => {
                log::error!("全ての設定の削除に失敗しました: {:?}", e);
                false
            }
        }
    } else {
        log::error!("ローカルストレージを取得できませんでした");
        false
    }
}

/// ローカルストレージを取得
fn get_local_storage() -> Option<Storage> {
    window()
        .and_then(|win| win.local_storage().ok())
        .flatten()
}

/// 型に対応する設定キーを取得
fn get_settings_key<T>() -> String
where
    T: 'static,
{
    let type_name = std::any::type_name::<T>();
    let key = if type_name.contains("AppSettings") {
        "remote-desktop.settings"
    } else {
        "remote-desktop.unknown"
    };
    
    key.to_string()
}

/// セッションストレージに一時的な値を保存
pub fn set_session_value(key: &str, value: &str) -> bool {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.session_storage() {
            if storage.set_item(key, value).is_ok() {
                return true;
            }
        }
    }
    false
}

/// セッションストレージから一時的な値を取得
pub fn get_session_value(key: &str) -> Option<String> {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.session_storage() {
            if let Ok(value) = storage.get_item(key) {
                return value;
            }
        }
    }
    None
}