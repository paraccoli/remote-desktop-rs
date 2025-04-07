//! エラー型定義
//!
//! リモートデスクトップアプリケーションで使用する共通エラー型を定義します。

use std::fmt;
use std::io;
use thiserror::Error;

/// 共通エラー
#[derive(Error, Debug)]
pub enum CommonError {
    /// 入出力エラー
    #[error("I/Oエラー: {0}")]
    IoError(#[from] io::Error),
    
    /// シリアライズエラー
    #[error("シリアライズエラー: {0}")]
    SerializeError(String),
    
    /// デシリアライズエラー
    #[error("デシリアライズエラー: {0}")]
    DeserializeError(String),
    
    /// ネットワークエラー
    #[error("ネットワークエラー: {0}")]
    NetworkError(String),
    
    /// 暗号化エラー
    #[error("暗号化エラー: {0}")]
    EncryptionError(String),
    
    /// 認証エラー
    #[error("認証エラー: {0}")]
    AuthenticationError(String),
    
    /// 設定エラー
    #[error("設定エラー: {0}")]
    ConfigError(String),
    
    /// タイムアウトエラー
    #[error("タイムアウト: {0}")]
    TimeoutError(String),
    
    /// ビジー状態
    #[error("ビジー状態: {0}")]
    BusyError(String),
    
    /// 無効なパラメータ
    #[error("無効なパラメータ: {0}")]
    InvalidParameterError(String),
    
    /// サポートされていない操作
    #[error("サポートされていない操作: {0}")]
    UnsupportedOperationError(String),
    
    /// リソースが見つからない
    #[error("リソースが見つかりません: {0}")]
    ResourceNotFoundError(String),
    
    /// その他のエラー
    #[error("{0}")]
    Other(String),
}

/// エラーコード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// 成功
    Success = 0,
    /// 一般的なエラー
    GeneralError = 1,
    /// I/Oエラー
    IoError = 2,
    /// シリアライズエラー
    SerializeError = 3,
    /// デシリアライズエラー
    DeserializeError = 4,
    /// ネットワークエラー
    NetworkError = 5,
    /// 暗号化エラー
    EncryptionError = 6,
    /// 認証エラー
    AuthenticationError = 7,
    /// 設定エラー
    ConfigError = 8,
    /// タイムアウト
    TimeoutError = 9,
    /// ビジー状態
    BusyError = 10,
    /// 無効なパラメータ
    InvalidParameterError = 11,
    /// サポートされていない操作
    UnsupportedOperationError = 12,
    /// リソースが見つからない
    ResourceNotFoundError = 13,
    /// その他のエラー
    Other = 99,
}

impl ErrorCode {
    /// エラーコードから文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::Success => "Success",
            ErrorCode::GeneralError => "GeneralError",
            ErrorCode::IoError => "IoError",
            ErrorCode::SerializeError => "SerializeError",
            ErrorCode::DeserializeError => "DeserializeError",
            ErrorCode::NetworkError => "NetworkError",
            ErrorCode::EncryptionError => "EncryptionError",
            ErrorCode::AuthenticationError => "AuthenticationError",
            ErrorCode::ConfigError => "ConfigError",
            ErrorCode::TimeoutError => "TimeoutError",
            ErrorCode::BusyError => "BusyError",
            ErrorCode::InvalidParameterError => "InvalidParameterError",
            ErrorCode::UnsupportedOperationError => "UnsupportedOperationError",
            ErrorCode::ResourceNotFoundError => "ResourceNotFoundError",
            ErrorCode::Other => "Other",
        }
    }
    
    /// 数値からエラーコードを取得
    pub fn from_i32(code: i32) -> Self {
        match code {
            0 => ErrorCode::Success,
            1 => ErrorCode::GeneralError,
            2 => ErrorCode::IoError,
            3 => ErrorCode::SerializeError,
            4 => ErrorCode::DeserializeError,
            5 => ErrorCode::NetworkError,
            6 => ErrorCode::EncryptionError,
            7 => ErrorCode::AuthenticationError,
            8 => ErrorCode::ConfigError,
            9 => ErrorCode::TimeoutError,
            10 => ErrorCode::BusyError,
            11 => ErrorCode::InvalidParameterError,
            12 => ErrorCode::UnsupportedOperationError,
            13 => ErrorCode::ResourceNotFoundError,
            _ => ErrorCode::Other,
        }
    }
}

/// エラー詳細
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    /// エラーコード
    pub code: ErrorCode,
    /// エラーメッセージ
    pub message: String,
    /// エラー発生時のコンテキスト情報
    pub context: Option<String>,
    /// エラー発生場所
    pub location: Option<String>,
    /// 原因となるエラー
    pub cause: Option<String>,
}

impl ErrorDetails {
    /// 新しいエラー詳細を作成
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: None,
            location: None,
            cause: None,
        }
    }
    
    /// コンテキスト情報を追加
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
    
    /// 発生場所情報を追加
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
    
    /// 原因情報を追加
    pub fn with_cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }
}

impl fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code.as_str(), self.message)?;
        
        if let Some(context) = &self.context {
            write!(f, " (Context: {})", context)?;
        }
        
        if let Some(location) = &self.location {
            write!(f, " at {}", location)?;
        }
        
        if let Some(cause) = &self.cause {
            write!(f, " caused by: {}", cause)?;
        }
        
        Ok(())
    }
}

/// 結果型のエイリアス
pub type Result<T> = std::result::Result<T, CommonError>;