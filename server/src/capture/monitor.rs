//! モニター情報モジュール
//!
//! システムに接続されているモニターの情報を取得・管理します。

use serde::{Serialize, Deserialize};
use std::fmt;

/// モニター情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    /// モニターインデックス
    pub index: usize,
    /// 位置 (x, y)
    pub position: (i32, i32),
    /// サイズ (width, height)
    pub size: (u32, u32),
    /// 作業領域サイズ (width, height) - タスクバーなどを除いた領域
    pub work_area: (u32, u32),
    /// プライマリモニターかどうか
    pub is_primary: bool,
    /// モニター名/識別子
    pub name: String,
    /// プラットフォーム固有のハンドル
    #[serde(skip)]
    pub handle: usize,
}

impl Monitor {
    /// 新しいモニター情報を作成
    pub fn new(
        index: usize,
        position: (i32, i32),
        size: (u32, u32),
        is_primary: bool,
        name: String,
    ) -> Self {
        Self {
            index,
            position,
            size,
            work_area: size, // デフォルトではサイズと同じ
            is_primary,
            name,
            handle: 0,
        }
    }
    
    /// モニターの解像度を取得
    pub fn resolution(&self) -> (u32, u32) {
        self.size
    }
    
    /// モニターの横幅を取得
    pub fn width(&self) -> u32 {
        self.size.0
    }
    
    /// モニターの高さを取得
    pub fn height(&self) -> u32 {
        self.size.1
    }
    
    /// モニターのX座標を取得
    pub fn x(&self) -> i32 {
        self.position.0
    }
    
    /// モニターのY座標を取得
    pub fn y(&self) -> i32 {
        self.position.1
    }
    
    /// モニターの名前を取得
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// モニターがプライマリかどうか
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }
    
    /// モニター情報を人間が読みやすい形式で取得
    pub fn display_info(&self) -> String {
        let primary_mark = if self.is_primary { "✓" } else { " " };
        format!(
            "Monitor #{} [{}]: {}x{} at ({},{}) - {}",
            self.index,
            primary_mark,
            self.size.0,
            self.size.1,
            self.position.0,
            self.position.1,
            self.name
        )
    }
}

impl fmt::Display for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let primary_mark = if self.is_primary { " (主)" } else { "" };
        write!(
            f,
            "モニター #{}{}: {}x{} @ ({},{})",
            self.index,
            primary_mark,
            self.size.0,
            self.size.1,
            self.position.0,
            self.position.1
        )
    }
}

/// モニター情報の概要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    /// モニター一覧
    pub monitors: Vec<Monitor>,
    /// プライマリモニターのインデックス
    pub primary_index: usize,
    /// 全モニターの合計領域
    pub total_area: (i32, i32, i32, i32), // (min_x, min_y, max_x, max_y)
}

impl MonitorInfo {
    /// モニター一覧から情報を構築
    pub fn from_monitors(monitors: Vec<Monitor>) -> Self {
        let primary_index = monitors.iter()
            .position(|m| m.is_primary)
            .unwrap_or(0);
        
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        
        for monitor in &monitors {
            min_x = min_x.min(monitor.position.0);
            min_y = min_y.min(monitor.position.1);
            max_x = max_x.max(monitor.position.0 + monitor.size.0 as i32);
            max_y = max_y.max(monitor.position.1 + monitor.size.1 as i32);
        }
        
        Self {
            monitors,
            primary_index,
            total_area: (min_x, min_y, max_x, max_y),
        }
    }
    
    /// モニターの数を取得
    pub fn count(&self) -> usize {
        self.monitors.len()
    }
    
    /// プライマリモニターを取得
    pub fn primary(&self) -> Option<&Monitor> {
        self.monitors.get(self.primary_index)
    }
    
    /// 指定されたインデックスのモニターを取得
    pub fn get(&self, index: usize) -> Option<&Monitor> {
        self.monitors.get(index)
    }
    
    /// 全モニターの合計幅を取得
    pub fn total_width(&self) -> i32 {
        self.total_area.2 - self.total_area.0
    }
    
    /// 全モニターの合計高さを取得
    pub fn total_height(&self) -> i32 {
        self.total_area.3 - self.total_area.1
    }
}