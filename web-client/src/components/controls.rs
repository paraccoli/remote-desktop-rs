//! コントロールパネルコンポーネント
//!
//! リモートデスクトップのコントロールを提供するUIコンポーネントです。

use wasm_bindgen::prelude::*;
use web_sys::{HtmlInputElement, Event};
use yew::prelude::*;
use crate::state::AppState;

/// コントロールパネルのプロパティ
#[derive(Properties, Clone, PartialEq)]
pub struct ControlPanelProps {
    /// 画質値
    pub quality: u8,
    /// FPS値
    pub fps: u8,
    /// 接続状態
    pub connected: bool,
    /// 画質変更ハンドラー
    pub on_quality_change: Callback<u8>,
    /// FPS変更ハンドラー
    pub on_fps_change: Callback<u8>,
    /// キーボード送信ハンドラー
    pub on_send_key_combo: Callback<Vec<String>>,
    /// クリップボード取得ハンドラー
    pub on_get_clipboard: Callback<()>,
    /// クリップボード設定ハンドラー
    pub on_set_clipboard: Callback<String>,
}

/// コントロールパネルコンポーネント
#[function_component(ControlPanel)]
pub fn control_panel(props: &ControlPanelProps) -> Html {
    // クリップボードテキスト用の状態
    let clipboard_text = use_state(|| String::new());
    
    // 画質変更ハンドラー
    let on_quality_change = {
        let on_quality_change = props.on_quality_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                if let Ok(quality) = input.value().parse::<u8>() {
                    on_quality_change.emit(quality);
                }
            }
        })
    };
    
    // FPS変更ハンドラー
    let on_fps_change = {
        let on_fps_change = props.on_fps_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                if let Ok(fps) = input.value().parse::<u8>() {
                    on_fps_change.emit(fps);
                }
            }
        })
    };
    
    // クリップボードテキスト変更ハンドラー
    let on_clipboard_text_change = {
        let clipboard_text = clipboard_text.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                clipboard_text.set(input.value());
            }
        })
    };
    
    // クリップボード取得ボタンクリックハンドラー
    let on_get_clipboard_click = {
        let on_get_clipboard = props.on_get_clipboard.clone();
        Callback::from(move |_| {
            on_get_clipboard.emit(());
        })
    };
    
    // クリップボード設定ボタンクリックハンドラー
    let on_set_clipboard_click = {
        let clipboard_text = clipboard_text.clone();
        let on_set_clipboard = props.on_set_clipboard.clone();
        Callback::from(move |_| {
            on_set_clipboard.emit((*clipboard_text).clone());
        })
    };
    
    // 特殊キー送信ハンドラー
    let on_send_special_key = {
        let on_send_key_combo = props.on_send_key_combo.clone();
        Callback::from(move |key: Vec<String>| {
            on_send_key_combo.emit(key);
        })
    };
    
    // Ctrl+Alt+Del キー送信
    let on_send_ctrl_alt_del = {
        let on_send_special_key = on_send_special_key.clone();
        Callback::from(move |_| {
            on_send_special_key.emit(vec!["Control".to_string(), "Alt".to_string(), "Delete".to_string()]);
        })
    };
    
    // Win+D キー送信
    let on_send_win_d = {
        let on_send_special_key = on_send_special_key.clone();
        Callback::from(move |_| {
            on_send_special_key.emit(vec!["Meta".to_string(), "d".to_string()]);
        })
    };
    
    // Alt+Tab キー送信
    let on_send_alt_tab = {
        let on_send_special_key = on_send_special_key.clone();
        Callback::from(move |_| {
            on_send_special_key.emit(vec!["Alt".to_string(), "Tab".to_string()]);
        })
    };
    
    // Alt+F4 キー送信
    let on_send_alt_f4 = {
        let on_send_special_key = on_send_special_key.clone();
        Callback::from(move |_| {
            on_send_special_key.emit(vec!["Alt".to_string(), "F4".to_string()]);
        })
    };
    
    // Esc キー送信
    let on_send_esc = {
        let on_send_special_key = on_send_special_key.clone();
        Callback::from(move |_| {
            on_send_special_key.emit(vec!["Escape".to_string()]);
        })
    };
    
    html! {
        <div class="control-panel">
            <h2>{"コントロール"}</h2>
            
            <div class="section">
                <h3>{"画質設定"}</h3>
                <div class="form-group">
                    <label for="quality-slider">{"画質: "}{props.quality}</label>
                    <input 
                        type="range" 
                        id="quality-slider" 
                        min="10" 
                        max="100" 
                        value={props.quality.to_string()} 
                        onchange={on_quality_change}
                        disabled={!props.connected}
                    />
                </div>
                
                <div class="form-group">
                    <label for="fps-slider">{"FPS: "}{props.fps}</label>
                    <input 
                        type="range" 
                        id="fps-slider" 
                        min="1" 
                        max="60" 
                        value={props.fps.to_string()} 
                        onchange={on_fps_change}
                        disabled={!props.connected}
                    />
                </div>
            </div>
            
            <div class="section">
                <h3>{"特殊キー"}</h3>
                <div class="key-buttons">
                    <button 
                        onclick={on_send_ctrl_alt_del} 
                        disabled={!props.connected}
                        title="Ctrl+Alt+Del"
                    >
                        {"Ctrl+Alt+Del"}
                    </button>
                    <button 
                        onclick={on_send_win_d} 
                        disabled={!props.connected}
                        title="Win+D (デスクトップ表示)"
                    >
                        {"Win+D"}
                    </button>
                    <button 
                        onclick={on_send_alt_tab} 
                        disabled={!props.connected}
                        title="Alt+Tab (ウィンドウ切替)"
                    >
                        {"Alt+Tab"}
                    </button>
                    <button 
                        onclick={on_send_alt_f4} 
                        disabled={!props.connected}
                        title="Alt+F4 (ウィンドウ閉じる)"
                    >
                        {"Alt+F4"}
                    </button>
                    <button 
                        onclick={on_send_esc} 
                        disabled={!props.connected}
                        title="Esc"
                    >
                        {"Esc"}
                    </button>
                </div>
            </div>
            
            <div class="section">
                <h3>{"クリップボード"}</h3>
                <div class="clipboard-controls">
                    <textarea
                        value={(*clipboard_text).clone()}
                        onchange={on_clipboard_text_change}
                        placeholder="クリップボードテキスト"
                        rows="4"
                        disabled={!props.connected}
                    />
                    <div class="clipboard-buttons">
                        <button 
                            onclick={on_get_clipboard_click} 
                            disabled={!props.connected}
                            title="リモートのクリップボードを取得"
                        >
                            {"取得"}
                        </button>
                        <button 
                            onclick={on_set_clipboard_click} 
                            disabled={!props.connected}
                            title="リモートのクリップボードに設定"
                        >
                            {"設定"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}