//! 入力イベント処理
//!
//! マウスやキーボードからの入力イベントを処理し、リモートサーバーに送信する
//! コマンドに変換する機能を提供します。

use super::{MouseButton, Modifier, InputMode, KeyMapping};
use crate::network::Command;
use egui::{Key, Pos2};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 入力イベントの種類
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// マウス移動
    MouseMove {
        /// 画面上の座標
        pos: Pos2,
        /// 相対移動量
        delta: Option<Pos2>,
    },
    /// マウスクリック
    MouseClick {
        /// ボタン
        button: MouseButton,
        /// ダブルクリックかどうか
        double: bool,
    },
    /// マウスボタン押下
    MouseDown {
        /// ボタン
        button: MouseButton,
    },
    /// マウスボタン解放
    MouseUp {
        /// ボタン
        button: MouseButton,
    },
    /// マウスホイールスクロール
    MouseScroll {
        /// X軸スクロール量
        delta_x: f32,
        /// Y軸スクロール量
        delta_y: f32,
    },
    /// キー押下
    KeyDown {
        /// キー
        key: Key,
        /// 修飾キー
        modifiers: Vec<Modifier>,
    },
    /// キー解放
    KeyUp {
        /// キー
        key: Key,
    },
    /// テキスト入力
    TextInput {
        /// 入力されたテキスト
        text: String,
    },
}

/// 入力イベントハンドラ
pub struct InputEventHandler {
    /// 入力モード
    mode: InputMode,
    /// 修飾キーの状態
    modifiers: HashMap<Modifier, bool>,
    /// 最後のマウス位置
    last_mouse_pos: Option<Pos2>,
    /// マウスボタンの状態
    mouse_button_state: HashMap<MouseButton, bool>,
    /// ダブルクリック検出用の最後のクリック時間
    last_click_time: HashMap<MouseButton, Instant>,
    /// ダブルクリックの時間しきい値
    double_click_threshold: Duration,
    /// キーマッピング
    key_mapping: KeyMapping,
    /// 入力コマンドのバッチ処理用キュー
    command_queue: Vec<Command>,
    /// キー状態
    key_state: HashMap<Key, bool>,
}

impl InputEventHandler {
    /// 新しい入力イベントハンドラを作成
    pub fn new() -> Self {
        Self {
            mode: InputMode::Normal,
            modifiers: HashMap::new(),
            last_mouse_pos: None,
            mouse_button_state: HashMap::new(),
            last_click_time: HashMap::new(),
            double_click_threshold: Duration::from_millis(500),
            key_mapping: KeyMapping::new(),
            command_queue: Vec::new(),
            key_state: HashMap::new(),
        }
    }

    /// 入力モードを設定
    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    /// 入力モードを取得
    pub fn mode(&self) -> InputMode {
        self.mode
    }

    /// マウス移動イベントを処理
    pub fn handle_mouse_move(&mut self, pos: Pos2) -> Option<Command> {
        if self.mode == InputMode::ViewOnly {
            return None;
        }

        let delta = self.last_mouse_pos.map(|last| Pos2::new(pos.x - last.x, pos.y - last.y));
        self.last_mouse_pos = Some(pos);

        if delta.map_or(true, |d| d.x.abs() > 0.5 || d.y.abs() > 0.5) {
            Some(Command::MouseMove {
                x: pos.x as i32,
                y: pos.y as i32,
            })
        } else {
            None
        }
    }

    /// マウスクリックイベントを処理
    pub fn handle_mouse_click(&mut self, button: MouseButton) -> Option<Command> {
        if self.mode == InputMode::ViewOnly {
            return None;
        }

        // ダブルクリック検出
        let now = Instant::now();
        let is_double_click = self
            .last_click_time
            .get(&button)
            .map_or(false, |last| now.duration_since(*last) < self.double_click_threshold);

        self.last_click_time.insert(button, now);

        if is_double_click {
            Some(Command::MouseClick {
                button,
                double: true,
            })
        } else {
            Some(Command::MouseClick {
                button,
                double: false,
            })
        }
    }

    /// マウスボタン押下イベントを処理
    pub fn handle_mouse_down(&mut self, button: MouseButton) -> Option<Command> {
        if self.mode == InputMode::ViewOnly {
            return None;
        }

        self.mouse_button_state.insert(button, true);
        Some(Command::MouseDown { button })
    }

    /// マウスボタン解放イベントを処理
    pub fn handle_mouse_up(&mut self, button: MouseButton) -> Option<Command> {
        if self.mode == InputMode::ViewOnly {
            return None;
        }

        self.mouse_button_state.insert(button, false);
        Some(Command::MouseUp { button })
    }

    /// マウスホイールスクロールイベントを処理
    pub fn handle_mouse_scroll(&mut self, delta_x: f32, delta_y: f32) -> Option<Command> {
        if self.mode == InputMode::ViewOnly {
            return None;
        }

        // 閾値を設定して微小なスクロールを無視
        if delta_x.abs() < 0.1 && delta_y.abs() < 0.1 {
            return None;
        }

        Some(Command::MouseScroll {
            delta_x: (delta_x * 10.0) as i32,
            delta_y: (delta_y * 10.0) as i32,
        })
    }

    /// キー押下イベントを処理
    pub fn handle_key_down(&mut self, key: Key) -> Option<Command> {
        if self.mode == InputMode::ViewOnly {
            return None;
        }

        // 修飾キーの状態を更新
        if let Some(modifier) = self.key_to_modifier(key) {
            self.modifiers.insert(modifier, true);
            return Some(Command::KeyDown {
                key: self.key_mapping.map_key(key),
            });
        }

        // 既に押されているキーは無視
        if self.key_state.get(&key).copied().unwrap_or(false) {
            return None;
        }

        self.key_state.insert(key, true);

        // 修飾キーとの組み合わせを考慮
        let active_modifiers: Vec<Modifier> = self
            .modifiers
            .iter()
            .filter_map(|(m, active)| if *active { Some(*m) } else { None })
            .collect();

        if !active_modifiers.is_empty() {
            let keys: Vec<Key> = active_modifiers
                .iter()
                .map(|m| match m {
                    Modifier::Ctrl => Key::Ctrl,
                    Modifier::Alt => Key::Alt,
                    Modifier::Shift => Key::Shift,
                    Modifier::Super => Key::Super,
                })
                .chain(std::iter::once(key))
                .collect();

            Some(Command::KeyCombo { keys })
        } else {
            Some(Command::KeyDown {
                key: self.key_mapping.map_key(key),
            })
        }
    }

    /// キー解放イベントを処理
    pub fn handle_key_up(&mut self, key: Key) -> Option<Command> {
        if self.mode == InputMode::ViewOnly {
            return None;
        }

        // 修飾キーの状態を更新
        if let Some(modifier) = self.key_to_modifier(key) {
            self.modifiers.insert(modifier, false);
            return Some(Command::KeyUp {
                key: self.key_mapping.map_key(key),
            });
        }

        self.key_state.insert(key, false);

        Some(Command::KeyUp {
            key: self.key_mapping.map_key(key),
        })
    }

    /// テキスト入力イベントを処理
    pub fn handle_text_input(&mut self, text: &str) -> Option<Command> {
        if self.mode == InputMode::ViewOnly || text.is_empty() {
            return None;
        }

        // 特殊文字の処理
        if text.len() == 1 {
            if let Some(key) = self.key_mapping.char_to_key(text.chars().next().unwrap()) {
                return Some(Command::KeyPress { key });
            }
        }

        // 通常のテキスト入力はキーのシーケンスとして処理
        let keys = text
            .chars()
            .filter_map(|c| self.key_mapping.char_to_key(c))
            .collect();

        Some(Command::KeySequence { keys })
    }

    /// キーからモディファイアへの変換
    fn key_to_modifier(&self, key: Key) -> Option<Modifier> {
        match key {
            Key::Ctrl => Some(Modifier::Ctrl),
            Key::Alt => Some(Modifier::Alt),
            Key::Shift => Some(Modifier::Shift),
            Key::Super => Some(Modifier::Super),
            _ => None,
        }
    }

    /// イベントを処理し、対応するコマンドを返す
    pub fn process_event(&mut self, event: InputEvent) -> Option<Command> {
        match event {
            InputEvent::MouseMove { pos, .. } => self.handle_mouse_move(pos),
            InputEvent::MouseClick { button, .. } => self.handle_mouse_click(button),
            InputEvent::MouseDown { button } => self.handle_mouse_down(button),
            InputEvent::MouseUp { button } => self.handle_mouse_up(button),
            InputEvent::MouseScroll { delta_x, delta_y } => self.handle_mouse_scroll(delta_x, delta_y),
            InputEvent::KeyDown { key, .. } => self.handle_key_down(key),
            InputEvent::KeyUp { key } => self.handle_key_up(key),
            InputEvent::TextInput { text } => self.handle_text_input(&text),
        }
    }

    /// 入力状態をリセット
    pub fn reset(&mut self) {
        self.modifiers.clear();
        self.last_mouse_pos = None;
        self.mouse_button_state.clear();
        self.key_state.clear();
        self.command_queue.clear();
    }
}

impl Default for InputEventHandler {
    fn default() -> Self {
        Self::new()
    }
}