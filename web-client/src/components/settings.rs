//! 設定パネルコンポーネント
//!
//! アプリケーション設定を表示・編集するUIコンポーネントです。

use wasm_bindgen::prelude::*;
use web_sys::{HtmlInputElement, Event};
use yew::prelude::*;
use serde::{Serialize, Deserialize};

/// アプリケーション設定
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// パフォーマンス優先モード
    pub performance_mode: bool,
    /// 自動再接続
    pub auto_reconnect: bool,
    /// 再接続の最大試行回数
    pub max_reconnect_attempts: u8,
    /// ポーリング間隔（ミリ秒）
    pub polling_interval: u32,
    /// 画質
    pub default_quality: u8,
    /// FPS上限
    pub max_fps: u8,
    /// WebRTC使用フラグ
    pub use_webrtc: bool,
    /// クリップボード同期
    pub sync_clipboard: bool,
    /// キーボードレイアウト
    pub keyboard_layout: String,
    /// デバッグモード
    pub debug_mode: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            performance_mode: false,
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            polling_interval: 100,
            default_quality: 50,
            max_fps: 30,
            use_webrtc: true,
            sync_clipboard: true,
            keyboard_layout: "auto".to_string(),
            debug_mode: false,
        }
    }
}

/// 設定パネルのプロパティ
#[derive(Properties, Clone, PartialEq)]
pub struct SettingsPanelProps {
    /// アプリケーション設定
    pub settings: AppSettings,
    /// 設定変更時のコールバック
    pub on_settings_change: Callback<AppSettings>,
}

/// 設定パネルコンポーネント
#[function_component(SettingsPanel)]
pub fn settings_panel(props: &SettingsPanelProps) -> Html {
    // 設定コピー
    let settings = use_state(|| props.settings.clone());
    
    // 性能優先モード変更ハンドラー
    let on_performance_mode_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_settings = (*settings).clone();
                new_settings.performance_mode = input.checked();
                settings.set(new_settings.clone());
                on_settings_change.emit(new_settings);
            }
        })
    };
    
    // 自動再接続変更ハンドラー
    let on_auto_reconnect_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_settings = (*settings).clone();
                new_settings.auto_reconnect = input.checked();
                settings.set(new_settings.clone());
                on_settings_change.emit(new_settings);
            }
        })
    };
    
    // 再接続試行回数変更ハンドラー
    let on_max_reconnect_attempts_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                if let Ok(value) = input.value().parse::<u8>() {
                    let mut new_settings = (*settings).clone();
                    new_settings.max_reconnect_attempts = value;
                    settings.set(new_settings.clone());
                    on_settings_change.emit(new_settings);
                }
            }
        })
    };
    
    // ポーリング間隔変更ハンドラー
    let on_polling_interval_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                if let Ok(value) = input.value().parse::<u32>() {
                    let mut new_settings = (*settings).clone();
                    new_settings.polling_interval = value;
                    settings.set(new_settings.clone());
                    on_settings_change.emit(new_settings);
                }
            }
        })
    };
    
    // 画質変更ハンドラー
    let on_default_quality_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                if let Ok(value) = input.value().parse::<u8>() {
                    let mut new_settings = (*settings).clone();
                    new_settings.default_quality = value;
                    settings.set(new_settings.clone());
                    on_settings_change.emit(new_settings);
                }
            }
        })
    };
    
    // FPS上限変更ハンドラー
    let on_max_fps_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                if let Ok(value) = input.value().parse::<u8>() {
                    let mut new_settings = (*settings).clone();
                    new_settings.max_fps = value;
                    settings.set(new_settings.clone());
                    on_settings_change.emit(new_settings);
                }
            }
        })
    };
    
    // WebRTC使用設定変更ハンドラー
    let on_use_webrtc_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_settings = (*settings).clone();
                new_settings.use_webrtc = input.checked();
                settings.set(new_settings.clone());
                on_settings_change.emit(new_settings);
            }
        })
    };
    
    // クリップボード同期変更ハンドラー
    let on_sync_clipboard_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_settings = (*settings).clone();
                new_settings.sync_clipboard = input.checked();
                settings.set(new_settings.clone());
                on_settings_change.emit(new_settings);
            }
        })
    };
    
    // キーボードレイアウト変更ハンドラー
    let on_keyboard_layout_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_settings = (*settings).clone();
                new_settings.keyboard_layout = input.value();
                settings.set(new_settings.clone());
                on_settings_change.emit(new_settings);
            }
        })
    };
    
    // デバッグモード変更ハンドラー
    let on_debug_mode_change = {
        let settings = settings.clone();
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_settings = (*settings).clone();
                new_settings.debug_mode = input.checked();
                settings.set(new_settings.clone());
                on_settings_change.emit(new_settings);
            }
        })
    };
    
    // 設定をリセットするハンドラー
    let on_reset_settings = {
        let on_settings_change = props.on_settings_change.clone();
        Callback::from(move |_| {
            let default_settings = AppSettings::default();
            on_settings_change.emit(default_settings);
        })
    };
    
    html! {
        <div class="settings-panel">
            <h2>{"アプリケーション設定"}</h2>
            
            <div class="settings-section">
                <h3>{"パフォーマンス"}</h3>
                
                <div class="form-group">
                    <label>
                        <input 
                            type="checkbox" 
                            checked={settings.performance_mode} 
                            onchange={on_performance_mode_change}
                        />
                        {"パフォーマンス優先モード"}
                    </label>
                    <div class="help-text">{"帯域幅を節約するために画質を落とします"}</div>
                </div>
                
                <div class="form-group">
                    <label for="default-quality">{"デフォルト画質: "}{settings.default_quality}</label>
                    <input 
                        type="range" 
                        id="default-quality" 
                        min="10" 
                        max="100" 
                        value={settings.default_quality.to_string()} 
                        onchange={on_default_quality_change}
                    />
                </div>
                
                <div class="form-group">
                    <label for="max-fps">{"FPS上限: "}{settings.max_fps}</label>
                    <input 
                        type="range" 
                        id="max-fps" 
                        min="1" 
                        max="60" 
                        value={settings.max_fps.to_string()} 
                        onchange={on_max_fps_change}
                    />
                </div>
                
                <div class="form-group">
                    <label for="polling-interval">{"更新間隔 (ms):"}</label>
                    <input 
                        type="number" 
                        id="polling-interval" 
                        value={settings.polling_interval.to_string()} 
                        onchange={on_polling_interval_change}
                        min="10" 
                        max="2000"
                    />
                    <div class="help-text">{"画面更新間隔 (ミリ秒)"}</div>
                </div>
            </div>
            
            <div class="settings-section">
                <h3>{"接続設定"}</h3>
                
                <div class="form-group">
                    <label>
                        <input 
                            type="checkbox" 
                            checked={settings.auto_reconnect} 
                            onchange={on_auto_reconnect_change}
                        />
                        {"自動再接続"}
                    </label>
                </div>
                
                <div class="form-group">
                    <label for="max-reconnect-attempts">{"最大再接続試行回数:"}</label>
                    <input 
                        type="number" 
                        id="max-reconnect-attempts" 
                        value={settings.max_reconnect_attempts.to_string()} 
                        onchange={on_max_reconnect_attempts_change}
                        min="1" 
                        max="20"
                    />
                </div>
                
                <div class="form-group">
                    <label>
                        <input 
                            type="checkbox" 
                            checked={settings.use_webrtc} 
                            onchange={on_use_webrtc_change}
                        />
                        {"WebRTCを使用 (可能な場合)"}
                    </label>
                    <div class="help-text">{"WebRTCが利用可能な場合に優先して使用します"}</div>
                </div>
            </div>
            
            <div class="settings-section">
                <h3>{"入出力設定"}</h3>
                
                <div class="form-group">
                    <label>
                        <input 
                            type="checkbox" 
                            checked={settings.sync_clipboard} 
                            onchange={on_sync_clipboard_change}
                        />
                        {"クリップボード同期"}
                    </label>
                </div>
                
                <div class="form-group">
                    <label for="keyboard-layout">{"キーボードレイアウト:"}</label>
                    <select 
                        id="keyboard-layout" 
                        value={settings.keyboard_layout.clone()} 
                        onchange={on_keyboard_layout_change}
                    >
                        <option value="auto">{"自動検出"}</option>
                        <option value="us">{"英語 (US)"}</option>
                        <option value="jp">{"日本語"}</option>
                        <option value="uk">{"英語 (UK)"}</option>
                        <option value="de">{"ドイツ語"}</option>
                        <option value="fr">{"フランス語"}</option>
                    </select>
                </div>
            </div>
            
            <div class="settings-section">
                <h3>{"詳細設定"}</h3>
                
                <div class="form-group">
                    <label>
                        <input 
                            type="checkbox" 
                            checked={settings.debug_mode} 
                            onchange={on_debug_mode_change}
                        />
                        {"デバッグモード"}
                    </label>
                    <div class="help-text">{"詳細なログ情報をコンソールに表示します"}</div>
                </div>
                
                <div class="form-actions">
                    <button onclick={on_reset_settings} class="reset-button">
                        {"設定をリセット"}
                    </button>
                </div>
            </div>
        </div>
    }
}