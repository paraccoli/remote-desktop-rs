//! リモートデスクトップ共通ライブラリ
//!
//! このクレートは、リモートデスクトップアプリケーションで使用される
//! 共通の機能を提供します。クライアントとサーバーの両方で使用されます。

pub mod compression;
pub mod config;
pub mod encryption;
pub mod error;
pub mod protocol;
pub mod utils;

// 主要コンポーネントを再エクスポート
pub use error::{CommonError, ErrorCode, ErrorDetails, Result};
pub use config::Config;
pub use protocol::{Command, Response, ConnectionInfo, ImageFormat};

/// ライブラリのバージョン
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// ビルド情報
pub struct BuildInfo {
    /// バージョン
    pub version: &'static str,
    /// ビルド日時
    pub build_date: &'static str,
    /// コミットハッシュ
    pub commit_hash: Option<&'static str>,
    /// Rustのバージョン
    pub rust_version: &'static str,
}

/// ビルド情報を取得
pub fn build_info() -> BuildInfo {
    // クロック時間から現在日時を取得
    let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    BuildInfo {
        version: VERSION,
        build_date: &*Box::leak(build_date.into_boxed_str()), // 静的ライフタイムに変換
        commit_hash: option_env!("GIT_HASH"),
        rust_version: &*Box::leak(format!("{}", rustc_version_runtime::version()).into_boxed_str()),
    }
}

/// ライブラリを初期化
pub fn initialize() -> Result<()> {
    // ロガーを初期化
    if let Err(e) = utils::logging::init_logger(
        utils::path::get_config_dir().join("logs").join("common.log"),
        utils::logging::LogLevel::Info
    ) {
        eprintln!("ロガーの初期化に失敗しました: {}", e);
    }
    
    // パニックハンドラを設定
    utils::logging::set_panic_hook();
    
    // バージョン情報をログに記録
    let build = build_info();
    let _ = utils::logging::info(&format!(
        "リモートデスクトップ共通ライブラリ初期化 - バージョン: {}, ビルド日時: {}, Rust: {}",
        build.version,
        build.build_date,
        build.rust_version
    ));
    
    Ok(())
}

/// 対応プラットフォームかどうかをチェック
pub fn check_platform_support() -> bool {
    cfg!(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    ))
}

/// プラットフォーム名を取得
pub fn get_platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
    }
}

/// アーキテクチャ名を取得
pub fn get_architecture_name() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "aarch64") {
        "ARM64"
    } else if cfg!(target_arch = "arm") {
        "ARM"
    } else {
        "Unknown"
    }
}