//! ロギング機能
//!
//! アプリケーションのロギング機能を提供します。

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;
use chrono::Local;
use lazy_static::lazy_static;

/// ログレベル
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// トレース情報
    Trace = 0,
    /// デバッグ情報
    Debug = 1,
    /// 一般情報
    Info = 2,
    /// 警告
    Warn = 3,
    /// エラー
    Error = 4,
    /// 致命的エラー
    Fatal = 5,
}

impl LogLevel {
    /// ログレベルを文字列に変換
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
    
    /// 文字列からログレベルを解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(LogLevel::Trace),
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" | "WARNING" => Some(LogLevel::Warn),
            "ERROR" | "ERR" => Some(LogLevel::Error),
            "FATAL" | "CRITICAL" => Some(LogLevel::Fatal),
            _ => None,
        }
    }
}

/// ファイルロガー
pub struct FileLogger {
    /// ログファイル
    file: Mutex<File>,
    /// 最小ログレベル
    min_level: LogLevel,
}

impl FileLogger {
    /// 新しいファイルロガーを作成
    pub fn new<P: AsRef<Path>>(path: P, min_level: LogLevel) -> io::Result<Self> {
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = path.as_ref().parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(path)?;
        
        Ok(Self {
            file: Mutex::new(file),
            min_level,
        })
    }
    
    /// ログを記録
    pub fn log(&self, level: LogLevel, message: &str) -> io::Result<()> {
        if level < self.min_level {
            return Ok(());
        }
        
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("{} [{}] {}\n", timestamp, level.as_str(), message);
        
        let mut file = self.file.lock().unwrap();
        file.write_all(log_line.as_bytes())?;
        file.flush()?;
        
        Ok(())
    }
    
    /// トレースログを記録
    pub fn trace(&self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Trace, message)
    }
    
    /// デバッグログを記録
    pub fn debug(&self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Debug, message)
    }
    
    /// 情報ログを記録
    pub fn info(&self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Info, message)
    }
    
    /// 警告ログを記録
    pub fn warn(&self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Warn, message)
    }
    
    /// エラーログを記録
    pub fn error(&self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Error, message)
    }
    
    /// 致命的エラーログを記録
    pub fn fatal(&self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Fatal, message)
    }
    
    /// 最小ログレベルを設定
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }
}

lazy_static! {
    /// グローバルロガー
    static ref GLOBAL_LOGGER: Mutex<Option<FileLogger>> = Mutex::new(None);
}

/// グローバルロガーを初期化
pub fn init_logger<P: AsRef<Path>>(path: P, min_level: LogLevel) -> io::Result<()> {
    let logger = FileLogger::new(path, min_level)?;
    let mut global_logger = GLOBAL_LOGGER.lock().unwrap();
    *global_logger = Some(logger);
    Ok(())
}

/// グローバルログを記録
pub fn log(level: LogLevel, message: &str) -> io::Result<()> {
    if let Some(logger) = &*GLOBAL_LOGGER.lock().unwrap() {
        logger.log(level, message)
    } else {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("{} [{}] {}", timestamp, level.as_str(), message);
        Ok(())
    }
}

/// トレースログを記録
pub fn trace(message: &str) -> io::Result<()> {
    log(LogLevel::Trace, message)
}

/// デバッグログを記録
pub fn debug(message: &str) -> io::Result<()> {
    log(LogLevel::Debug, message)
}

/// 情報ログを記録
pub fn info(message: &str) -> io::Result<()> {
    log(LogLevel::Info, message)
}

/// 警告ログを記録
pub fn warn(message: &str) -> io::Result<()> {
    log(LogLevel::Warn, message)
}

/// エラーログを記録
pub fn error(message: &str) -> io::Result<()> {
    log(LogLevel::Error, message)
}

/// 致命的エラーログを記録
pub fn fatal(message: &str) -> io::Result<()> {
    log(LogLevel::Fatal, message)
}

/// パニック時のログ記録ハンドラーを設定
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let message = match panic_info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => s.as_str(),
                None => "Unknown panic payload",
            },
        };
        
        let location = match panic_info.location() {
            Some(loc) => format!(" at {}:{}", loc.file(), loc.line()),
            None => String::new(),
        };
        
        let _ = fatal(&format!("Panic: {}{}", message, location));
    }));
}