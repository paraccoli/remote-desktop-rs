//! フォーマットユーティリティ
//!
//! 数値や日時などをフォーマットするユーティリティ関数を提供します。

use wasm_bindgen::prelude::*;
use std::fmt;

/// バイト数を人間が読みやすい形式にフォーマット
///
/// # 例
///
/// ```
/// let bytes = 1024 * 1024 * 3.5;
/// let formatted = format_bytes(bytes);
/// assert_eq!(formatted, "3.5 MB");
/// ```
pub fn format_bytes(bytes: f64) -> String {
    if bytes < 1024.0 {
        format!("{:.0} B", bytes)
    } else if bytes < 1024.0 * 1024.0 {
        format!("{:.1} KB", bytes / 1024.0)
    } else if bytes < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
    }
}

/// 秒数を時間:分:秒形式にフォーマット
///
/// # 例
///
/// ```
/// let seconds = 3661; // 1時間1分1秒
/// let formatted = format_time(seconds);
/// assert_eq!(formatted, "1:01:01");
/// ```
pub fn format_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    format!("{}:{:02}:{:02}", hours, minutes, secs)
}

/// 秒数を稼働時間の形式（日、時間、分、秒）でフォーマット
///
/// # 例
///
/// ```
/// let seconds = 86400 + 3600 + 60 + 5; // 1日1時間1分5秒
/// let formatted = format_uptime(seconds);
/// assert_eq!(formatted, "1日 1時間 1分 5秒");
/// ```
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    let mut result = String::new();
    
    if days > 0 {
        result.push_str(&format!("{}日 ", days));
    }
    
    if hours > 0 || days > 0 {
        result.push_str(&format!("{}時間 ", hours));
    }
    
    if minutes > 0 || hours > 0 || days > 0 {
        result.push_str(&format!("{}分 ", minutes));
    }
    
    result.push_str(&format!("{}秒", secs));
    
    result
}

/// 日時をフォーマット
pub fn format_datetime(timestamp: f64) -> String {
    use js_sys::Date;
    
    let date = Date::new(&JsValue::from_f64(timestamp));
    
    let year = date.get_full_year();
    let month = date.get_month() + 1; // JavaScriptの月は0始まり
    let day = date.get_date();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    let seconds = date.get_seconds();
    
    format!("{}/{:02}/{:02} {:02}:{:02}:{:02}", 
        year, month, day, hours, minutes, seconds)
}

/// 表形式のデータを整形
pub fn format_table<T, F>(data: &[T], headers: &[&str], row_formatter: F) -> String
where
    F: Fn(&T) -> Vec<String>,
{
    if data.is_empty() {
        return String::new();
    }
    
    // ヘッダー行を追加
    let mut rows = vec![headers.iter().map(|s| s.to_string()).collect::<Vec<_>>()];
    
    // データ行を追加
    for item in data {
        rows.push(row_formatter(item));
    }
    
    // 各列の最大幅を計算
    let col_count = headers.len();
    let mut col_widths = vec![0; col_count];
    
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                col_widths[i] = std::cmp::max(col_widths[i], cell.len());
            }
        }
    }
    
    // 表を構築
    let mut result = String::new();
    
    // ヘッダー行
    for (i, cell) in rows[0].iter().enumerate() {
        if i > 0 {
            result.push_str(" | ");
        }
        result.push_str(&format!("{:width$}", cell, width = col_widths[i]));
    }
    result.push('\n');
    
    // 区切り線
    for (i, &width) in col_widths.iter().enumerate() {
        if i > 0 {
            result.push_str("-+-");
        }
        result.push_str(&"-".repeat(width));
    }
    result.push('\n');
    
    // データ行
    for row in rows.iter().skip(1) {
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                result.push_str(" | ");
            }
            if i < col_count {
                result.push_str(&format!("{:width$}", cell, width = col_widths[i]));
            }
        }
        result.push('\n');
    }
    
    result
}