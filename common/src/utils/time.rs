//! 時間ユーティリティ
//!
//! 時間処理に関連するユーティリティ機能を提供します。

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, Local}; // NaiveDateTimeは使用されていないため削除

/// 現在のUNIXタイムスタンプ（ミリ秒）を取得
pub fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as u64
}

/// 現在のUNIXタイムスタンプ（秒）を取得
pub fn current_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

/// タイムアウト処理を提供するラッパー
pub struct Timeout {
    /// 開始時刻
    start: Instant,
    /// タイムアウト時間
    duration: Duration,
}

impl Timeout {
    /// 新しいタイムアウトを作成
    pub fn new(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }
    
    /// 指定されたミリ秒でタイムアウトを作成
    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }
    
    /// タイムアウトしたかどうか確認
    pub fn is_elapsed(&self) -> bool {
        self.start.elapsed() >= self.duration
    }
    
    /// タイムアウトまでの残り時間を取得
    pub fn remaining(&self) -> Duration {
        if self.is_elapsed() {
            Duration::from_secs(0)
        } else {
            self.duration - self.start.elapsed()
        }
    }
    
    /// タイムアウトをリセット
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
    
    /// タイムアウト時間を変更
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }
}

/// タイムスタンプをフォーマット
pub fn format_timestamp(timestamp_millis: u64) -> String {
    let seconds = (timestamp_millis / 1000) as i64;
    let nanos = ((timestamp_millis % 1000) * 1_000_000) as u32;
      match DateTime::<Utc>::from_timestamp(seconds, nanos) {
        Some(dt) => {
            let local_dt = dt.with_timezone(&Local);
            local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
        },
        None => "Invalid timestamp".to_string(),
    }
}

/// UNIXタイムスタンプを日時文字列に変換
pub fn unix_timestamp_to_string(timestamp_secs: u64) -> String {    match DateTime::<Utc>::from_timestamp(timestamp_secs as i64, 0) {
        Some(dt) => {
            let local_dt = dt.with_timezone(&Local);
            local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
        },
        None => "Invalid timestamp".to_string(),
    }
}

/// 現在時刻のISO 8601形式文字列を取得
pub fn current_time_iso8601() -> String {
    Utc::now().to_rfc3339()
}

/// 経過時間を測定するタイマー
pub struct Timer {
    /// 開始時刻
    start: Instant,
    /// ラップタイム
    laps: Vec<Duration>,
}

impl Timer {
    /// 新しいタイマーを作成して開始
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
            laps: Vec::new(),
        }
    }
    
    /// 経過時間を取得
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    
    /// 経過時間をミリ秒で取得
    pub fn elapsed_millis(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }
    
    /// ラップタイムを記録
    pub fn lap(&mut self) -> Duration {
        let elapsed = self.elapsed();
        self.laps.push(elapsed);
        elapsed
    }
    
    /// ラップタイムのリストを取得
    pub fn laps(&self) -> &[Duration] {
        &self.laps
    }
    
    /// タイマーをリセット
    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.laps.clear();
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::start()
    }
}

/// スリープ関数のユーティリティラッパー
pub fn sleep(duration: Duration) {
    std::thread::sleep(duration);
}

/// ミリ秒指定でスリープ
pub fn sleep_millis(millis: u64) {
    sleep(Duration::from_millis(millis));
}