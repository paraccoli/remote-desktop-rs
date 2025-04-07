//! ロギングユーティリティ
//!
//! WebAssemblyからJavaScriptのコンソールにログを出力するユーティリティ関数を提供します。

use wasm_bindgen::prelude::*;
use web_sys::console;

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    /// ログレベルを文字列に変換
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// デバッグログ
pub fn log_debug(message: &str) {
    log(LogLevel::Debug, message);
}

/// 情報ログ
pub fn log_info(message: &str) {
    log(LogLevel::Info, message);
}

/// 警告ログ
pub fn log_warning(message: &str) {
    log(LogLevel::Warning, message);
}

/// エラーログ
pub fn log_error(message: &str) {
    log(LogLevel::Error, message);
}

/// ログを出力
pub fn log(level: LogLevel, message: &str) {
    let formatted = format!("[{}] {}", level.as_str(), message);
    
    match level {
        LogLevel::Debug => console::debug_1(&JsValue::from_str(&formatted)),
        LogLevel::Info => console::info_1(&JsValue::from_str(&formatted)),
        LogLevel::Warning => console::warn_1(&JsValue::from_str(&formatted)),
        LogLevel::Error => console::error_1(&JsValue::from_str(&formatted)),
    }
}

/// オブジェクトをコンソールに出力
pub fn log_object(level: LogLevel, label: &str, object: &JsValue) {
    let prefix = format!("[{}] {}: ", level.as_str(), label);
    
    match level {
        LogLevel::Debug => {
            console::debug_2(
                &JsValue::from_str(&prefix),
                object
            )
        },
        LogLevel::Info => {
            console::info_2(
                &JsValue::from_str(&prefix),
                object
            )
        },
        LogLevel::Warning => {
            console::warn_2(
                &JsValue::from_str(&prefix),
                object
            )
        },
        LogLevel::Error => {
            console::error_2(
                &JsValue::from_str(&prefix),
                object
            )
        },
    }
}

/// パフォーマンス計測開始
pub fn start_performance_measurement(name: &str) {
    console::time_with_label(name);
}

/// パフォーマンス計測終了と結果表示
pub fn end_performance_measurement(name: &str) {
    console::time_end_with_label(name);
}

/// グループログ開始
pub fn start_log_group(name: &str) {
    console::log(&format!("Group: {}", name));
}

/// グループログ終了
pub fn end_log_group() {
    console::group_end();
}

/// ロググループ内でコード実行
pub fn with_log_group<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    start_log_group(name);
    let result = f();
    end_log_group();
    result
}

/// パフォーマンス計測付きでコード実行
pub fn with_performance_measurement<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    start_performance_measurement(name);
    let result = f();
    end_performance_measurement(name);
    result
}