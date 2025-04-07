//! ユーティリティモジュール
//!
//! 各種ユーティリティ機能を提供します。

pub mod time;
pub mod logging;

/// パス関連のユーティリティ
pub mod path {
    use std::path::{Path, PathBuf};
    
    /// 設定ディレクトリを取得
    pub fn get_config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("remote-desktop-rs")
    }
    
    /// キャッシュディレクトリを取得
    pub fn get_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("remote-desktop-rs")
    }
    
    /// 実行可能ファイルのディレクトリからの相対パスを解決
    pub fn resolve_from_exe(relative_path: &str) -> Option<PathBuf> {
        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;
        Some(exe_dir.join(relative_path))
    }
    
    /// パスが存在するか確認し、ディレクトリなら作成
    pub fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        } else if !path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Path exists but is not a directory: {:?}", path),
            ));
        }
        Ok(())
    }
}

/// 文字列関連のユーティリティ
pub mod string {
    /// 文字列を安全に切り詰める
    pub fn truncate_str(s: &str, max_chars: usize) -> String {
        if s.chars().count() <= max_chars {
            return s.to_string();
        }
        
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if i >= max_chars - 3 {
                break;
            }
            result.push(c);
        }
        result.push_str("...");
        result
    }
    
    /// 複数行の文字列をエスケープする
    pub fn escape_multiline(s: &str) -> String {
        s.replace('\n', "\\n").replace('\r', "\\r")
    }
    
    /// 行を結合する
    pub fn join_lines(lines: &[&str]) -> String {
        lines.join("\n")
    }
}

/// システム関連のユーティリティ
pub mod system {
    use std::time::Duration;
    use sysinfo::{SystemExt, CpuExt};
    
    /// システム情報
    #[derive(Debug, Clone)]
    pub struct SystemInfo {
        /// CPU使用率 (0.0-100.0)
        pub cpu_usage: f32,
        /// メモリ使用率 (0.0-100.0)
        pub memory_usage: f32,
        /// 合計メモリ (バイト)
        pub total_memory: u64,
        /// 使用中メモリ (バイト)
        pub used_memory: u64,
        /// 稼働時間
        pub uptime: Duration,
    }
    
    /// システム情報を取得
    #[cfg(target_os = "linux")]
    pub fn get_system_info() -> SystemInfo {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_usage = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };
        let uptime = Duration::from_secs(sys.uptime());
        
        SystemInfo {
            cpu_usage,
            memory_usage,
            total_memory,
            used_memory,
            uptime,
        }
    }
    
    #[cfg(target_os = "windows")]
    pub fn get_system_info() -> SystemInfo {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_usage = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };
        let uptime = Duration::from_secs(sys.uptime());
        
        SystemInfo {
            cpu_usage,
            memory_usage,
            total_memory,
            used_memory,
            uptime,
        }
    }
    
    #[cfg(target_os = "macos")]
    pub fn get_system_info() -> SystemInfo {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_usage = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };
        let uptime = Duration::from_secs(sys.uptime());
        
        SystemInfo {
            cpu_usage,
            memory_usage,
            total_memory,
            used_memory,
            uptime,
        }
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    pub fn get_system_info() -> SystemInfo {
        SystemInfo {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            total_memory: 0,
            used_memory: 0,
            uptime: Duration::from_secs(0),
        }
    }
}

/// 数値関連のユーティリティ
pub mod number {
    /// 範囲内にクランプする
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
    
    /// バイト単位を人間が読みやすい形式に変換
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
        
        if bytes == 0 {
            return "0 B".to_string();
        }
        
        let bytes_f64 = bytes as f64;
        let exponent = (bytes_f64.ln() / 1024_f64.ln()).floor() as usize;
        let exponent = exponent.min(UNITS.len() - 1);
        
        let value = bytes_f64 / 1024_f64.powi(exponent as i32);
        
        if exponent == 0 {
            format!("{} {}", value as u64, UNITS[exponent])
        } else {
            format!("{:.2} {}", value, UNITS[exponent])
        }
    }
    
    /// 時間をフォーマット (秒 -> HH:MM:SS)
    pub fn format_duration(seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let seconds = seconds % 60;
        
        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

/// エラー処理関連のユーティリティ
pub mod error {
    use std::fmt;
    
    /// エラー詳細情報
    #[derive(Debug)]
    pub struct ErrorDetails {
        /// エラーコード
        pub code: i32,
        /// エラーメッセージ
        pub message: String,
        /// エラー詳細
        pub details: Option<String>,
        /// エラー発生場所
        pub location: Option<String>,
    }
    
    impl ErrorDetails {
        /// 新しいエラー詳細を作成
        pub fn new<S: Into<String>>(code: i32, message: S) -> Self {
            Self {
                code,
                message: message.into(),
                details: None,
                location: None,
            }
        }
        
        /// 詳細を追加
        pub fn with_details<S: Into<String>>(mut self, details: S) -> Self {
            self.details = Some(details.into());
            self
        }
        
        /// 場所を追加
        pub fn with_location<S: Into<String>>(mut self, location: S) -> Self {
            self.location = Some(location.into());
            self
        }
    }
    
    impl fmt::Display for ErrorDetails {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Error {}: {}", self.code, self.message)?;
            
            if let Some(details) = &self.details {
                write!(f, " - {}", details)?;
            }
            
            if let Some(location) = &self.location {
                write!(f, " at {}", location)?;
            }
            
            Ok(())
        }
    }
}