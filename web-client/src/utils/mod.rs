//! ユーティリティモジュール
//!
//! このモジュールには、Webクライアントで使用される様々なユーティリティ関数や構造体が含まれています。

pub mod format;
pub mod storage;
pub mod network;
pub mod logging;
pub mod clipboard;

// 主要ユーティリティを再エクスポート
pub use format::{format_bytes, format_time, format_uptime};
pub use storage::{load_settings, save_settings};
pub use logging::{log_debug, log_info, log_warning, log_error};