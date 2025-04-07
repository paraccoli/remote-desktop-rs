//! 接続コンポーネント
//!
//! サーバーへの接続フォームを提供するコンポーネントです。

use wasm_bindgen::prelude::*;
use web_sys::{HtmlInputElement, Event};
use yew::prelude::*;
use serde::{Serialize, Deserialize};
use crate::state::{ConnectionInfo, AppState};

/// 接続フォームのプロパティ
#[derive(Properties, Clone, PartialEq)]
pub struct ConnectionFormProps {
    /// 接続状態
    pub connected: bool,
    /// 接続情報
    pub connection_info: ConnectionInfo,
    /// 接続ハンドラー
    pub on_connect: Callback<ConnectionInfo>,
    /// 切断ハンドラー
    pub on_disconnect: Callback<()>,
}

/// 接続フォームの状態
#[derive(Clone, PartialEq)]
struct ConnectionFormState {
    /// ホスト名
    host: String,
    /// ポート番号
    port: u16,
    /// 接続プロトコル
    protocol: String,
    /// TLSを使用するか
    use_tls: bool,
    /// WebRTCを優先するか
    prefer_webrtc: bool,
    /// ユーザー名
    username: String,
    /// パスワード
    password: String,
    /// 接続エラーメッセージ
    error_message: Option<String>,
}

/// 接続フォームコンポーネント
#[function_component(ConnectionForm)]
pub fn connection_form(props: &ConnectionFormProps) -> Html {
    // 内部状態を初期化
    let state = use_state(|| ConnectionFormState {
        host: props.connection_info.host.clone(),
        port: props.connection_info.port,
        protocol: props.connection_info.protocol.clone(),
        use_tls: props.connection_info.use_tls,
        prefer_webrtc: props.connection_info.prefer_webrtc,
        username: props.connection_info.username.clone().unwrap_or_default(),
        password: props.connection_info.password.clone().unwrap_or_default(),
        error_message: None,
    });

    // ホスト名が変更された時のハンドラー
    let on_host_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_state = (*state).clone();
                new_state.host = input.value();
                state.set(new_state);
            }
        })
    };

    // ポート番号が変更された時のハンドラー
    let on_port_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                if let Ok(port) = input.value().parse::<u16>() {
                    let mut new_state = (*state).clone();
                    new_state.port = port;
                    state.set(new_state);
                }
            }
        })
    };

    // プロトコルが変更された時のハンドラー
    let on_protocol_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_state = (*state).clone();
                new_state.protocol = input.value();
                state.set(new_state);
            }
        })
    };

    // TLS使用設定が変更された時のハンドラー
    let on_tls_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_state = (*state).clone();
                new_state.use_tls = input.checked();
                state.set(new_state);
            }
        })
    };

    // WebRTC優先設定が変更された時のハンドラー
    let on_webrtc_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_state = (*state).clone();
                new_state.prefer_webrtc = input.checked();
                state.set(new_state);
            }
        })
    };

    // ユーザー名が変更された時のハンドラー
    let on_username_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_state = (*state).clone();
                new_state.username = input.value();
                state.set(new_state);
            }
        })
    };

    // パスワードが変更された時のハンドラー
    let on_password_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut new_state = (*state).clone();
                new_state.password = input.value();
                state.set(new_state);
            }
        })
    };

    // 接続ボタンがクリックされた時のハンドラー
    let on_connect_click = {
        let state = state.clone();
        let on_connect = props.on_connect.clone();
        Callback::from(move |_| {
            let connection_info = ConnectionInfo {
                host: state.host.clone(),
                port: state.port,
                protocol: state.protocol.clone(),
                use_tls: state.use_tls,
                prefer_webrtc: state.prefer_webrtc,
                username: if state.username.is_empty() { None } else { Some(state.username.clone()) },
                password: if state.password.is_empty() { None } else { Some(state.password.clone()) },
            };
            on_connect.emit(connection_info);
        })
    };

    // 切断ボタンがクリックされた時のハンドラー
    let on_disconnect_click = {
        let on_disconnect = props.on_disconnect.clone();
        Callback::from(move |_| {
            on_disconnect.emit(());
        })
    };

    html! {
        <div class="connection-form">
            <h2>{"サーバー接続"}</h2>
            
            <div class="form-group">
                <label for="host">{"ホスト:"}</label>
                <input 
                    type="text" 
                    id="host" 
                    value={state.host.clone()} 
                    onchange={on_host_change}
                    disabled={props.connected}
                />
            </div>
            
            <div class="form-group">
                <label for="port">{"ポート:"}</label>
                <input 
                    type="number" 
                    id="port" 
                    value={state.port.to_string()} 
                    onchange={on_port_change} 
                    min="1" 
                    max="65535"
                    disabled={props.connected}
                />
            </div>
            
            <div class="form-group">
                <label>{"接続方式:"}</label>
                <div class="radio-group">
                    <label>
                        <input 
                            type="radio" 
                            name="protocol" 
                            value="tcp" 
                            checked={state.protocol == "tcp"} 
                            onchange={on_protocol_change.clone()}
                            disabled={props.connected}
                        />
                        {"TCP"}
                    </label>
                    <label>
                        <input 
                            type="radio" 
                            name="protocol" 
                            value="websocket" 
                            checked={state.protocol == "websocket"} 
                            onchange={on_protocol_change.clone()}
                            disabled={props.connected}
                        />
                        {"WebSocket"}
                    </label>
                    <label>
                        <input 
                            type="radio" 
                            name="protocol" 
                            value="webrtc" 
                            checked={state.protocol == "webrtc"} 
                            onchange={on_protocol_change}
                            disabled={props.connected}
                        />
                        {"WebRTC"}
                    </label>
                </div>
            </div>
            
            <div class="form-group">
                <label>
                    <input 
                        type="checkbox" 
                        checked={state.use_tls} 
                        onchange={on_tls_change}
                        disabled={props.connected}
                    />
                    {"TLSを使用する"}
                </label>
            </div>
            
            <div class="form-group">
                <label>
                    <input 
                        type="checkbox" 
                        checked={state.prefer_webrtc} 
                        onchange={on_webrtc_change}
                        disabled={props.connected}
                    />
                    {"可能ならWebRTCを使用する"}
                </label>
            </div>
            
            <div class="form-group">
                <label for="username">{"ユーザー名:"}</label>
                <input 
                    type="text" 
                    id="username" 
                    value={state.username.clone()} 
                    onchange={on_username_change}
                    disabled={props.connected}
                />
            </div>
            
            <div class="form-group">
                <label for="password">{"パスワード:"}</label>
                <input 
                    type="password" 
                    id="password" 
                    value={state.password.clone()} 
                    onchange={on_password_change}
                    disabled={props.connected}
                />
            </div>
            
            {
                if let Some(error) = &state.error_message {
                    html! {
                        <div class="error-message">
                            {error}
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            
            <div class="form-actions">
                {
                    if props.connected {
                        html! {
                            <button onclick={on_disconnect_click} class="disconnect-button">
                                {"切断"}
                            </button>
                        }
                    } else {
                        html! {
                            <button onclick={on_connect_click} class="connect-button">
                                {"接続"}
                            </button>
                        }
                    }
                }
            </div>
        </div>
    }
}