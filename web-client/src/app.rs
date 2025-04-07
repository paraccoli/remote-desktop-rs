//! Webクライアントアプリケーション
//!
//! Yewを使用したWebクライアントのメインアプリケーションを実装します。

use wasm_bindgen::prelude::*;
use yew::prelude::*;
use std::rc::Rc;

use crate::components::{ConnectionForm, ControlPanel, RemoteDisplay, SettingsPanel, StatusBar};
use crate::components::display::{MouseEventInfo, KeyEventInfo};
use crate::components::status::{PerformanceInfo, SystemInfo, ConnectionStatus};
use crate::components::settings::AppSettings;
use crate::state::{AppState, ConnectionInfo};
use crate::utils::{storage, logging, network, format};
use crate::utils::network::{get_data_channel_state, RTC_DATA_CHANNEL_OPEN};

/// メインアプリケーション
#[function_component(App)]
pub fn app() -> Html {
    // 初期設定を読み込む
    let settings = use_state(|| {
        storage::load_settings::<AppSettings>().unwrap_or_default()
    });
    
    // 接続状態の管理
    let connection_state = use_state(|| ConnectionStatus::default());
    
    // 接続情報の管理
    let connection_info = use_state(|| ConnectionInfo {
        host: "localhost".to_string(),
        port: 9090,
        protocol: "websocket".to_string(),
        use_tls: false,
        prefer_webrtc: true,
        username: None,
        password: None,
    });
    
    // パフォーマンス情報の管理
    let performance = use_state(|| PerformanceInfo::default());
    
    // システム情報の管理
    let system_info = use_state(|| None::<SystemInfo>);
    
    // 画像データの管理
    let image_data = use_state(|| None::<String>);
    let image_width = use_state(|| None::<u32>);
    let image_height = use_state(|| None::<u32>);
    
    // 詳細表示の状態管理
    let show_details = use_state(|| false);
    
    // 設定パネル表示の状態管理
    let show_settings = use_state(|| false);
    
    // WebSocketとWebRTCの接続情報
    let websocket = use_state(|| None::<web_sys::WebSocket>);
    let webrtc = use_state(|| None::<(web_sys::RtcPeerConnection, web_sys::RtcDataChannel)>);
    
    // 最後のポーリング時間
    let last_polling_time = use_state(|| 0.0);
    
    // 統計情報
    let stats = use_state(|| {
        (0u64, 0u64) // (受信バイト数, 送信バイト数)
    });
    
    // エラーメッセージの状態管理
    let error_message = use_state(|| None::<String>);
    
    // 初期化処理
    use_effect(move || {
        logging::log_info("Webクライアントが初期化されました");
        
        // WebRTCサポートのチェック
        let has_webrtc = network::is_webrtc_supported();
        logging::log_info(&format!("WebRTCサポート: {}", has_webrtc));
        
        // 何もしないクリーンアップ関数
        || {}
    });
    
    // 接続ハンドラー
    let on_connect = {
        let connection_info = connection_info.clone();
        let connection_state = connection_state.clone();
        let websocket = websocket.clone();
        let webrtc = webrtc.clone();
        let error_message = error_message.clone();
        let settings = settings.clone();
        
        Callback::from(move |info: ConnectionInfo| {
            // 接続情報を更新
            connection_info.set(info.clone());
            
            // 接続状態を「接続中」に更新
            connection_state.set(ConnectionStatus {
                connected: false,
                connection_type: "接続中".to_string(),
                server_address: format!("{}:{}", info.host, info.port),
                using_tls: info.use_tls,
                connection_time: 0,
                status_message: "接続中...".to_string(),
            });
            
            // エラーメッセージをクリア
            error_message.set(None);
            
            // WebSocketまたはWebRTCで接続を試みる
            if info.prefer_webrtc && network::is_webrtc_supported() {
                // WebRTC接続を確立
                // 実際の実装では複雑になるため、このサンプルでは省略
                logging::log_info("WebRTCでの接続を試みています...");
                
                // 未実装のためエラー表示
                error_message.set(Some("WebRTC接続はまだ実装されていません".to_string()));
            } else {
                // WebSocket URL を構築
                let protocol = if info.use_tls { "wss" } else { "ws" };
                let ws_url = format!("{}://{}:{}/ws", protocol, info.host, info.port);
                
                logging::log_info(&format!("WebSocketでの接続を試みています: {}", ws_url));
                
                // WebSocketを作成
                match web_sys::WebSocket::new(&ws_url) {
                    Ok(ws) => {
                        // イベントハンドラーを設定
                        let onopen_callback = Closure::wrap(Box::new(move |_| {
                            logging::log_info("WebSocket接続が確立されました");
                            
                            // WebSocket接続が確立された後の処理は別途実装が必要
                        }) as Box<dyn FnMut(JsValue)>);
                        
                        let onerror_callback = {
                            let error_message = error_message.clone();
                            Closure::wrap(Box::new(move |e: JsValue| {
                                logging::log_error(&format!("WebSocket接続エラー: {:?}", e));
                                error_message.set(Some("WebSocket接続に失敗しました".to_string()));
                            }) as Box<dyn FnMut(JsValue)>)
                        };
                        
                        let onclose_callback = {
                            let connection_state = connection_state.clone();
                            Closure::wrap(Box::new(move |_| {
                                logging::log_info("WebSocket接続が閉じられました");
                                
                                // 接続状態を更新
                                let mut current = (*connection_state).clone();
                                current.connected = false;
                                current.status_message = "切断されました".to_string();
                                connection_state.set(current);
                            }) as Box<dyn FnMut(JsValue)>)
                        };
                        
                        // コールバックを設定
                        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                        
                        // WebSocketを保存
                        websocket.set(Some(ws));
                        
                        // メモリリークを防ぐためにコールバックを忘れないようにする
                        onopen_callback.forget();
                        onerror_callback.forget();
                        onclose_callback.forget();
                    },
                    Err(e) => {
                        logging::log_error(&format!("WebSocketの作成に失敗しました: {:?}", e));
                        error_message.set(Some("WebSocketの作成に失敗しました".to_string()));
                    }
                }
            }
        })
    };
    
    // 切断ハンドラー
    let on_disconnect = {
        let connection_state = connection_state.clone();
        let websocket = websocket.clone();
        let webrtc = webrtc.clone();
        
        Callback::from(move |_| {
            // WebSocketを閉じる
            if let Some(ws) = &*websocket {
                ws.close().ok();
                websocket.set(None);
            }
            
            // WebRTCを閉じる
            if let Some((peer_connection, data_channel)) = &*webrtc {
                data_channel.close();
                peer_connection.close();
                webrtc.set(None);
            }
            
            // 接続状態を更新
            connection_state.set(ConnectionStatus::default());
            
            logging::log_info("接続を切断しました");
        })
    };
    
    // 画質変更ハンドラー
    let on_quality_change = {
        let websocket = websocket.clone();
        let webrtc = webrtc.clone();
        let performance = performance.clone();
        
        Callback::from(move |quality: u8| {
            logging::log_info(&format!("画質を変更: {}", quality));
            
            // 画質設定コマンドを作成
            let command = format!(r#"{{"type":"SetQuality","quality":{}}}"#, quality);
            
            // WebSocketが接続されている場合
            if let Some(ws) = &*websocket {
                if ws.ready_state() == web_sys::WebSocket::OPEN {
                    // コマンドを送信
                    if let Err(e) = ws.send_with_str(&command) {
                        logging::log_error(&format!("コマンド送信エラー: {:?}", e));
                    }
                }
            }
            
            // WebRTCが接続されている場合
            if let Some((_, data_channel)) = &*webrtc {
                if get_data_channel_state(data_channel) == RTC_DATA_CHANNEL_OPEN {
                    // コマンドを送信
                    if let Err(e) = data_channel.send_with_str(&command) {
                        logging::log_error(&format!("コマンド送信エラー: {:?}", e));
                    }
                }
            }
            
            // パフォーマンス情報を更新
            let mut current_perf = (*performance).clone();
            current_perf.quality = quality;
            performance.set(current_perf);
        })
    };
    
    // FPS変更ハンドラー
    let on_fps_change = {
        let settings = settings.clone();
        
        Callback::from(move |fps: u8| {
            logging::log_info(&format!("FPS設定を変更: {}", fps));
            
            // 設定を更新
            let mut new_settings = (*settings).clone();
            new_settings.max_fps = fps;
            settings.set(new_settings.clone());
            
            // 設定を保存
            storage::save_settings(&new_settings);
        })
    };
    
    // マウスイベントハンドラー
    let on_mouse_event = {
        let websocket = websocket.clone();
        let webrtc = webrtc.clone();
        
        Callback::from(move |event: MouseEventInfo| {
            // マウスイベントコマンドを作成
            let command = match event.event_type.as_str() {
                "down" => {
                    if let Some(button) = event.button {
                        let button_str = match button {
                            crate::components::display::MouseButton::Left => "left",
                            crate::components::display::MouseButton::Middle => "middle",
                            crate::components::display::MouseButton::Right => "right",
                        };
                        format!(r#"{{"type":"MouseDown","button":"{}","x":{},"y":{}}}"#, 
                            button_str, event.x, event.y)
                    } else {
                        return;
                    }
                },
                "up" => {
                    if let Some(button) = event.button {
                        let button_str = match button {
                            crate::components::display::MouseButton::Left => "left",
                            crate::components::display::MouseButton::Middle => "middle",
                            crate::components::display::MouseButton::Right => "right",
                        };
                        format!(r#"{{"type":"MouseUp","button":"{}","x":{},"y":{}}}"#, 
                            button_str, event.x, event.y)
                    } else {
                        return;
                    }
                },
                "move" => {
                    format!(r#"{{"type":"MouseMove","x":{},"y":{}}}"#, event.x, event.y)
                },
                "wheel" => {
                    if let (Some(delta_x), Some(delta_y)) = (event.delta_x, event.delta_y) {
                        format!(r#"{{"type":"MouseScroll","delta_x":{},"delta_y":{}}}"#, 
                            delta_x, delta_y)
                    } else {
                        return;
                    }
                },
                _ => return,
            };
            
            // コマンドを送信
            if let Some(ws) = &*websocket {
                if ws.ready_state() == web_sys::WebSocket::OPEN {
                    if let Err(e) = ws.send_with_str(&command) {
                        logging::log_error(&format!("マウスコマンド送信エラー: {:?}", e));
                    }
                }
            }
            
            if let Some((_, data_channel)) = &*webrtc {
                if get_data_channel_state(data_channel) == RTC_DATA_CHANNEL_OPEN {
                    if let Err(e) = data_channel.send_with_str(&command) {
                        logging::log_error(&format!("マウスコマンド送信エラー: {:?}", e));
                    }
                }
            }
        })
    };
    
    // キーボードイベントハンドラー
    let on_key_event = {
        let websocket = websocket.clone();
        let webrtc = webrtc.clone();
        
        Callback::from(move |event: KeyEventInfo| {
            // キーボードイベントコマンドを作成
            let command = match event.event_type.as_str() {
                "down" => {
                    let modifiers_json = serde_json::to_string(&event.modifiers).unwrap_or_else(|_| "[]".to_string());
                    format!(r#"{{"type":"KeyDown","key_code":"{}","modifiers":{}}}"#, 
                        event.key_code, modifiers_json)
                },
                "up" => {
                    let modifiers_json = serde_json::to_string(&event.modifiers).unwrap_or_else(|_| "[]".to_string());
                    format!(r#"{{"type":"KeyUp","key_code":"{}","modifiers":{}}}"#, 
                        event.key_code, modifiers_json)
                },
                _ => return,
            };
            
            // コマンドを送信
            if let Some(ws) = &*websocket {
                if ws.ready_state() == web_sys::WebSocket::OPEN {
                    if let Err(e) = ws.send_with_str(&command) {
                        logging::log_error(&format!("キーボードコマンド送信エラー: {:?}", e));
                    }
                }
            }
            
            if let Some((_, data_channel)) = &*webrtc {
                if get_data_channel_state(data_channel) == RTC_DATA_CHANNEL_OPEN {
                    if let Err(e) = data_channel.send_with_str(&command) {
                        logging::log_error(&format!("キーボードコマンド送信エラー: {:?}", e));
                    }
                }
            }
        })
    };
    
    // 特殊キー送信ハンドラー
    let on_send_key_combo = {
        let on_key_event = on_key_event.clone();
        
        Callback::from(move |keys: Vec<String>| {
            for key in &keys {
                // キーダウンイベントを送信
                on_key_event.emit(KeyEventInfo {
                    event_type: "down".to_string(),
                    key_code: key.clone(),
                    modifiers: vec![],
                });
            }
            
            // 少し待ってからキーアップイベントを送信（本来はタイマーを使うべき）
            for key in keys.iter().rev() {
                on_key_event.emit(KeyEventInfo {
                    event_type: "up".to_string(),
                    key_code: key.clone(),
                    modifiers: vec![],
                });
            }
        })
    };
    
    // クリップボード取得ハンドラー
    let on_get_clipboard = {
        let websocket = websocket.clone();
        let webrtc = webrtc.clone();
        
        Callback::from(move |_| {
            // クリップボード取得コマンドを作成
            let command = r#"{"type":"GetClipboard"}"#;
            
            // コマンドを送信
            if let Some(ws) = &*websocket {
                if ws.ready_state() == web_sys::WebSocket::OPEN {
                    if let Err(e) = ws.send_with_str(command) {
                        logging::log_error(&format!("クリップボード取得コマンド送信エラー: {:?}", e));
                    }
                }
            }
            
            if let Some((_, data_channel)) = &*webrtc {
                if get_data_channel_state(data_channel) == RTC_DATA_CHANNEL_OPEN {
                    if let Err(e) = data_channel.send_with_str(command) {
                        logging::log_error(&format!("クリップボード取得コマンド送信エラー: {:?}", e));
                    }
                }
            }
        })
    };
    
    // クリップボード設定ハンドラー
    let on_set_clipboard = {
        let websocket = websocket.clone();
        let webrtc = webrtc.clone();
        
        Callback::from(move |text: String| {
            // クリップボード設定コマンドを作成
            let text_json = serde_json::to_string(&text).unwrap_or_else(|_| "\"\"".to_string());
            let command = format!(r#"{{"type":"SetClipboard","text":{}}}"#, text_json);
            
            // コマンドを送信
            if let Some(ws) = &*websocket {
                if ws.ready_state() == web_sys::WebSocket::OPEN {
                    if let Err(e) = ws.send_with_str(&command) {
                        logging::log_error(&format!("クリップボード設定コマンド送信エラー: {:?}", e));
                    }
                }
            }
            
            if let Some((_, data_channel)) = &*webrtc {
                if get_data_channel_state(data_channel) == RTC_DATA_CHANNEL_OPEN {
                    if let Err(e) = data_channel.send_with_str(&command) {
                        logging::log_error(&format!("クリップボード設定コマンド送信エラー: {:?}", e));
                    }
                }
            }
        })
    };
    
    // 設定変更ハンドラー
    let on_settings_change = {
        let settings = settings.clone();
        
        Callback::from(move |new_settings: AppSettings| {
            // 設定を更新
            settings.set(new_settings.clone());
            
            // 設定を保存
            storage::save_settings(&new_settings);
        })
    };
    
    // 詳細表示切り替えハンドラー
    let on_toggle_details = {
        let show_details = show_details.clone();
        
        Callback::from(move |_| {
            show_details.set(!*show_details);
        })
    };
    
    // 設定パネル表示切り替えハンドラー
    let on_toggle_settings = {
        let show_settings = show_settings.clone();
        
        Callback::from(move |_| {
            show_settings.set(!*show_settings);
        })
    };
    
    html! {
        <div class="app-container">
            <header class="app-header">
                <div class="logo">
                    <h1>{"リモートデスクトップ"}</h1>
                </div>
                <div class="header-controls">
                    <button onclick={on_toggle_settings.clone()} title="設定">
                        <i class="icon-settings"></i>
                    </button>
                </div>
            </header>
            
            <div class="main-container">
                <div class="sidebar">
                    <ConnectionForm
                        connected={connection_state.connected}
                        connection_info={(*connection_info).clone()}
                        on_connect={on_connect}
                        on_disconnect={on_disconnect}
                    />
                    
                    <ControlPanel
                        quality={performance.quality}
                        fps={settings.max_fps}
                        connected={connection_state.connected}
                        on_quality_change={on_quality_change}
                        on_fps_change={on_fps_change}
                        on_send_key_combo={on_send_key_combo}
                        on_get_clipboard={on_get_clipboard}
                        on_set_clipboard={on_set_clipboard}
                    />
                </div>
                
                <div class="display-container">
                    <RemoteDisplay
                        image_data={(*image_data).clone()}
                        width={*image_width}
                        height={*image_height}
                        connected={connection_state.connected}
                        on_mouse_event={on_mouse_event}
                        on_key_event={on_key_event}
                    />
                    
                    if let Some(error) = &*error_message {
                        <div class="error-overlay">
                            <div class="error-message">
                                <h3>{"エラー"}</h3>
                                <p>{error}</p>
                            </div>
                        </div>
                    }
                </div>
            </div>
            
            <footer class="app-footer">
                <StatusBar
                    performance={(*performance).clone()}
                    system_info={(*system_info).clone()}
                    connection={(*connection_state).clone()}
                    show_details={*show_details}
                    on_toggle_details={on_toggle_details}
                />
            </footer>
            
            if *show_settings {
                <div class="settings-overlay">
                    <div class="settings-panel-container">
                        <div class="settings-header">
                            <h2>{"設定"}</h2>
                            <button class="close-button" onclick={on_toggle_settings}>{"×"}</button>
                        </div>
                        <SettingsPanel
                            settings={(*settings).clone()}
                            on_settings_change={on_settings_change}
                        />
                    </div>
                </div>
            }
        </div>
    }
}