//! 設定管理
//!
//! アプリケーション設定の読み込み、保存、および管理機能を提供します。

use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::collections::HashMap;
use thiserror::Error;

/// 設定エラー
#[derive(Error, Debug)]
pub enum ConfigError {
    /// I/O エラー
    #[error("設定の読み書き中にI/Oエラーが発生しました: {0}")]
    IoError(#[from] io::Error),
    
    /// JSON エラー
    #[error("JSONの解析に失敗しました: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// TOML デシリアライズエラー
    #[error("TOMLの解析に失敗しました: {0}")]
    TomlDeError(#[from] toml::de::Error),
    
    /// TOML シリアライズエラー
    #[error("TOMLのシリアライズに失敗しました: {0}")]
    TomlSerError(#[from] toml::ser::Error),
    
    /// 設定キーが存在しない
    #[error("設定キー '{0}' が存在しません")]
    KeyNotFound(String),
    
    /// 型変換エラー
    #[error("設定値の型変換に失敗しました: {0}")]
    TypeError(String),
    
    /// その他のエラー
    #[error("設定エラー: {0}")]
    Other(String),
}

/// 設定形式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON 形式
    Json,
    /// TOML 形式
    Toml,
}

impl Default for ConfigFormat {
    fn default() -> Self {
        ConfigFormat::Json
    }
}

impl ConfigFormat {
    /// ファイル拡張子から設定形式を判定
    pub fn from_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "json" => Some(ConfigFormat::Json),
                "toml" => Some(ConfigFormat::Toml),
                _ => None,
            })
    }
    
    /// ファイル名から設定形式を判定
    pub fn from_path(path: &Path) -> Option<Self> {
        Self::from_extension(path)
    }
}

/// 設定値
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// 文字列値
    String(String),
    /// 整数値
    Integer(i64),
    /// 浮動小数点値
    Float(f64),
    /// 真偽値
    Boolean(bool),
    /// 配列
    Array(Vec<ConfigValue>),
    /// オブジェクト
    Object(HashMap<String, ConfigValue>),
    /// null 値
    Null,
}

impl ConfigValue {
    /// 文字列に変換
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }    /// 文字列に変換（エラー発生時に指定されたデフォルト値を返す）
    pub fn as_string_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.as_string().unwrap_or(default)
    }
    
    /// 整数に変換
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            ConfigValue::Float(f) => Some(*f as i64),
            ConfigValue::String(s) => s.parse::<i64>().ok(),
            _ => None,
        }
    }
    
    /// 整数に変換（エラー発生時に指定されたデフォルト値を返す）
    pub fn as_integer_or(&self, default: i64) -> i64 {
        self.as_integer().unwrap_or(default)
    }
    
    /// 浮動小数点数に変換
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Integer(i) => Some(*i as f64),
            ConfigValue::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }
    
    /// 浮動小数点数に変換（エラー発生時に指定されたデフォルト値を返す）
    pub fn as_float_or(&self, default: f64) -> f64 {
        self.as_float().unwrap_or(default)
    }
    
    /// 真偽値に変換
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            ConfigValue::Integer(i) => Some(*i != 0),
            ConfigValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "1" | "on" => Some(true),
                "false" | "no" | "0" | "off" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
    
    /// 真偽値に変換（エラー発生時に指定されたデフォルト値を返す）
    pub fn as_boolean_or(&self, default: bool) -> bool {
        self.as_boolean().unwrap_or(default)
    }
    
    /// 配列に変換
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(a) => Some(a),
            _ => None,
        }
    }
    
    /// オブジェクトに変換
    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(o) => Some(o),
            _ => None,
        }
    }
    
    /// 指定されたパスの値を取得
    pub fn get(&self, path: &str) -> Option<&ConfigValue> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = self;
        
        for part in parts {
            match current {
                ConfigValue::Object(obj) => {
                    current = obj.get(part)?;
                },
                _ => return None,
            }
        }
        
        Some(current)
    }
}

/// 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 設定値
    values: HashMap<String, ConfigValue>,
    /// 設定ファイルパス
    #[serde(skip)]
    path: Option<PathBuf>,
    /// 設定形式
    #[serde(skip)]
    format: ConfigFormat,
    /// 自動保存フラグ
    #[serde(skip)]
    auto_save: bool,
}

impl Config {
    /// 空の設定を作成
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            path: None,
            format: ConfigFormat::Json,
            auto_save: false,
        }
    }
      /// ファイルから設定を読み込み
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let format = ConfigFormat::from_path(path)
            .unwrap_or(ConfigFormat::Json);
        
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let values = match format {
            ConfigFormat::Json => serde_json::from_str(&content)?,
            ConfigFormat::Toml => toml::from_str(&content)?,
        };
        
        Ok(Self {
            values,
            path: Some(path.to_path_buf()),
            format,
            auto_save: false,
        })
    }
    
    /// 設定をファイルに保存
    pub fn save<P: AsRef<Path>>(&self, path: Option<P>) -> Result<(), ConfigError> {
        let path = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => match &self.path {
                Some(p) => p.clone(),
                None => return Err(ConfigError::Other("保存パスが指定されていません".to_string())),
            },
        };
        
        // ディレクトリが存在することを確認
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        let format = ConfigFormat::from_path(&path)
            .unwrap_or(self.format);
          let content = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(&self.values)?,
            ConfigFormat::Toml => toml::to_string(&self.values)?,
        };
        
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
        
        Ok(())
    }
    
    /// 自動保存を有効化
    pub fn enable_auto_save(&mut self) {
        self.auto_save = true;
    }
    
    /// 自動保存を無効化
    pub fn disable_auto_save(&mut self) {
        self.auto_save = false;
    }
    
    /// 設定パスを設定
    pub fn set_path<P: AsRef<Path>>(&mut self, path: P) {
        self.path = Some(path.as_ref().to_path_buf());
        
        if let Some(format) = ConfigFormat::from_path(path.as_ref()) {
            self.format = format;
        }
    }
    
    /// 設定形式を設定
    pub fn set_format(&mut self, format: ConfigFormat) {
        self.format = format;
    }
    
    /// 値を設定
    pub fn set<T: Into<ConfigValue>>(&mut self, key: &str, value: T) -> Result<(), ConfigError> {
        self.values.insert(key.to_string(), value.into());
        
        if self.auto_save {
            self.save::<PathBuf>(None)?;
        }
        
        Ok(())
    }
    
    /// 値を取得
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.values.get(key)
    }
    
    /// 値を削除
    pub fn remove(&mut self, key: &str) -> Result<Option<ConfigValue>, ConfigError> {
        let result = self.values.remove(key);
        
        if self.auto_save {
            self.save::<PathBuf>(None)?;
        }
        
        Ok(result)
    }
    
    /// 全ての設定キーを取得
    pub fn keys(&self) -> Vec<&String> {
        self.values.keys().collect()
    }
    
    /// 設定が含まれるかどうかを確認
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    /// 設定をマージ
    pub fn merge(&mut self, other: &Config) -> Result<(), ConfigError> {
        for (key, value) in &other.values {
            self.values.insert(key.clone(), value.clone());
        }
        
        if self.auto_save {
            self.save::<PathBuf>(None)?;
        }
        
        Ok(())
    }
    
    /// 文字列として値を取得
    pub fn get_string(&self, key: &str) -> Result<String, ConfigError> {
        match self.get(key) {
            Some(ConfigValue::String(s)) => Ok(s.clone()),
            Some(value) => match value {
                ConfigValue::Integer(i) => Ok(i.to_string()),
                ConfigValue::Float(f) => Ok(f.to_string()),
                ConfigValue::Boolean(b) => Ok(b.to_string()),
                _ => Err(ConfigError::TypeError(format!("値 '{}' を文字列に変換できません", key))),
            },
            None => Err(ConfigError::KeyNotFound(key.to_string())),
        }
    }
    
    /// 文字列として値を取得（キーが存在しない場合はデフォルト値を返す）
    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        self.get_string(key).unwrap_or_else(|_| default.to_string())
    }
    
    /// 整数として値を取得
    pub fn get_integer(&self, key: &str) -> Result<i64, ConfigError> {
        match self.get(key) {
            Some(value) => value
                .as_integer()
                .ok_or_else(|| ConfigError::TypeError(format!("値 '{}' を整数に変換できません", key))),
            None => Err(ConfigError::KeyNotFound(key.to_string())),
        }
    }
    
    /// 整数として値を取得（キーが存在しない場合はデフォルト値を返す）
    pub fn get_integer_or(&self, key: &str, default: i64) -> i64 {
        self.get_integer(key).unwrap_or(default)
    }
    
    /// 浮動小数点数として値を取得
    pub fn get_float(&self, key: &str) -> Result<f64, ConfigError> {
        match self.get(key) {
            Some(value) => value
                .as_float()
                .ok_or_else(|| ConfigError::TypeError(format!("値 '{}' を浮動小数点数に変換できません", key))),
            None => Err(ConfigError::KeyNotFound(key.to_string())),
        }
    }
    
    /// 浮動小数点数として値を取得（キーが存在しない場合はデフォルト値を返す）
    pub fn get_float_or(&self, key: &str, default: f64) -> f64 {
        self.get_float(key).unwrap_or(default)
    }
    
    /// 真偽値として値を取得
    pub fn get_boolean(&self, key: &str) -> Result<bool, ConfigError> {
        match self.get(key) {
            Some(value) => value
                .as_boolean()
                .ok_or_else(|| ConfigError::TypeError(format!("値 '{}' を真偽値に変換できません", key))),
            None => Err(ConfigError::KeyNotFound(key.to_string())),
        }
    }
    
    /// 真偽値として値を取得（キーが存在しない場合はデフォルト値を返す）
    pub fn get_boolean_or(&self, key: &str, default: bool) -> bool {
        self.get_boolean(key).unwrap_or(default)
    }
    
    /// 配列として値を取得
    pub fn get_array(&self, key: &str) -> Result<Vec<ConfigValue>, ConfigError> {
        match self.get(key) {
            Some(ConfigValue::Array(a)) => Ok(a.clone()),
            Some(_) => Err(ConfigError::TypeError(format!("値 '{}' は配列ではありません", key))),
            None => Err(ConfigError::KeyNotFound(key.to_string())),
        }
    }
    
    /// オブジェクトとして値を取得
    pub fn get_object(&self, key: &str) -> Result<HashMap<String, ConfigValue>, ConfigError> {
        match self.get(key) {
            Some(ConfigValue::Object(o)) => Ok(o.clone()),
            Some(_) => Err(ConfigError::TypeError(format!("値 '{}' はオブジェクトではありません", key))),
            None => Err(ConfigError::KeyNotFound(key.to_string())),
        }
    }
    
    /// 環境変数から設定を読み込み
    pub fn from_env(prefix: &str) -> Self {
        let mut config = Self::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let config_key = key[prefix.len()..].to_lowercase();
                config.values.insert(config_key, ConfigValue::String(value));
            }
        }
        
        config
    }
    
    /// 値を取得し、指定された型に変換
    pub fn get_as<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.get(key) {
            Some(value) => {
                let json = serde_json::to_value(value)
                    .map_err(|e| ConfigError::JsonError(e))?;
                
                serde_json::from_value(json)
                    .map_err(|e| ConfigError::JsonError(e))
            },
            None => Err(ConfigError::KeyNotFound(key.to_string())),
        }
    }
    
    /// 値を取得し、指定された型に変換（キーが存在しない場合はデフォルト値を返す）
    pub fn get_as_or<T>(&self, key: &str, default: T) -> T
    where
        T: for<'de> Deserialize<'de> + Clone,
    {
        self.get_as(key).unwrap_or_else(|_| default)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for ConfigValue {
    fn from(value: String) -> Self {
        ConfigValue::String(value)
    }
}

impl From<&str> for ConfigValue {
    fn from(value: &str) -> Self {
        ConfigValue::String(value.to_string())
    }
}

impl From<i64> for ConfigValue {
    fn from(value: i64) -> Self {
        ConfigValue::Integer(value)
    }
}

impl From<i32> for ConfigValue {
    fn from(value: i32) -> Self {
        ConfigValue::Integer(value as i64)
    }
}

impl From<u32> for ConfigValue {
    fn from(value: u32) -> Self {
        ConfigValue::Integer(value as i64)
    }
}

impl From<f64> for ConfigValue {
    fn from(value: f64) -> Self {
        ConfigValue::Float(value)
    }
}

impl From<f32> for ConfigValue {
    fn from(value: f32) -> Self {
        ConfigValue::Float(value as f64)
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        ConfigValue::Boolean(value)
    }
}

impl<T> From<Vec<T>> for ConfigValue
where
    T: Into<ConfigValue>,
{
    fn from(value: Vec<T>) -> Self {
        ConfigValue::Array(value.into_iter().map(Into::into).collect())
    }
}

impl<K, V> From<HashMap<K, V>> for ConfigValue
where
    K: ToString,
    V: Into<ConfigValue>,
{
    fn from(value: HashMap<K, V>) -> Self {
        let mut map = HashMap::new();
        for (k, v) in value {
            map.insert(k.to_string(), v.into());
        }
        ConfigValue::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_value_conversions() {
        // 文字列
        let value = ConfigValue::String("123".to_string());
        assert_eq!(value.as_string(), Some("123"));
        assert_eq!(value.as_integer(), Some(123));
        assert_eq!(value.as_float(), Some(123.0));
        
        // 整数
        let value = ConfigValue::Integer(456);
        assert_eq!(value.as_string(), None);
        assert_eq!(value.as_integer(), Some(456));
        assert_eq!(value.as_float(), Some(456.0));
        
        // 浮動小数点
        let value = ConfigValue::Float(3.14);
        assert_eq!(value.as_string(), None);
        assert_eq!(value.as_integer(), Some(3));
        assert_eq!(value.as_float(), Some(3.14));
        
        // 真偽値
        let value = ConfigValue::Boolean(true);
        assert_eq!(value.as_string(), None);
        assert_eq!(value.as_integer(), None);
        assert_eq!(value.as_float(), None);
        assert_eq!(value.as_boolean(), Some(true));
        
        // 文字列の真偽値
        let value = ConfigValue::String("true".to_string());
        assert_eq!(value.as_boolean(), Some(true));
        
        let value = ConfigValue::String("yes".to_string());
        assert_eq!(value.as_boolean(), Some(true));
        
        let value = ConfigValue::String("false".to_string());
        assert_eq!(value.as_boolean(), Some(false));
    }
    
    #[test]
    fn test_config_set_get() {
        let mut config = Config::new();
        
        // 文字列
        config.set("string", "hello").unwrap();
        assert_eq!(config.get_string("string").unwrap(), "hello");
        
        // 整数
        config.set("integer", 42).unwrap();
        assert_eq!(config.get_integer("integer").unwrap(), 42);
        
        // 浮動小数点
        config.set("float", 3.14).unwrap();
        assert_eq!(config.get_float("float").unwrap(), 3.14);
        
        // 真偽値
        config.set("boolean", true).unwrap();
        assert_eq!(config.get_boolean("boolean").unwrap(), true);
        
        // デフォルト値
        assert_eq!(config.get_string_or("non_existent", "default"), "default");
        assert_eq!(config.get_integer_or("non_existent", 100), 100);
        assert_eq!(config.get_float_or("non_existent", 1.23), 1.23);
        assert_eq!(config.get_boolean_or("non_existent", false), false);
    }
}