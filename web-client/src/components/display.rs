//! リモート画面表示コンポーネント
//!
//! リモートデスクトップの画面を表示するコンポーネントです。

use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, KeyboardEvent, WheelEvent, TouchEvent};
use yew::prelude::*;
use std::rc::Rc;

/// マウスボタンの種類
#[derive(Clone, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// マウスイベント情報
#[derive(Clone, Debug, PartialEq)]
pub struct MouseEventInfo {
    /// イベントの種類
    pub event_type: String,
    /// マウスボタン
    pub button: Option<MouseButton>,
    /// X座標
    pub x: i32,
    /// Y座標
    pub y: i32,
    /// ホイールのデルタX
    pub delta_x: Option<i32>,
    /// ホイールのデルタY
    pub delta_y: Option<i32>,
}

/// キーボードイベント情報
#[derive(Clone, Debug, PartialEq)]
pub struct KeyEventInfo {
    /// イベントの種類
    pub event_type: String,
    /// キーコード
    pub key_code: String,
    /// 修飾キー
    pub modifiers: Vec<String>,
}

/// リモート画面のプロパティ
#[derive(Properties, Clone, PartialEq)]
pub struct RemoteDisplayProps {
    /// 画像データ (Base64エンコード)
    pub image_data: Option<String>,
    /// 画像の幅
    pub width: Option<u32>,
    /// 画像の高さ
    pub height: Option<u32>,
    /// 接続状態
    pub connected: bool,
    /// マウスイベント時のコールバック
    pub on_mouse_event: Callback<MouseEventInfo>,
    /// キーボードイベント時のコールバック
    pub on_key_event: Callback<KeyEventInfo>,
}

/// リモート画面表示コンポーネント
#[function_component(RemoteDisplay)]
pub fn remote_display(props: &RemoteDisplayProps) -> Html {
    // キャンバス要素への参照
    let canvas_ref = use_node_ref();
    
    // 最後のマウス位置を追跡
    let last_mouse_pos = use_state(|| (0, 0));
    
    // キャンバスのフォーカス状態
    let is_focused = use_state(|| false);
    
    // マウスダウンイベントハンドラー
    let on_mouse_down = {
        let on_mouse_event = props.on_mouse_event.clone();
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |e: MouseEvent| {
            if !props.connected {
                return;
            }
            
            e.prevent_default();
            
            // マウスボタンを判定
            let button = match e.button() {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                _ => MouseButton::Left,
            };
            
            // キャンバス内の座標を計算
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let rect = canvas.get_bounding_client_rect();
                let x = (e.client_x() as f64 - rect.left()) as i32;
                let y = (e.client_y() as f64 - rect.top()) as i32;
                
                // スケーリング計算（表示サイズとリモート画面サイズの比率）
                let (scaled_x, scaled_y) = calculate_scaled_position(
                    x, y, canvas.width(), canvas.height(), props.width, props.height
                );
                
                on_mouse_event.emit(MouseEventInfo {
                    event_type: "down".to_string(),
                    button: Some(button),
                    x: scaled_x,
                    y: scaled_y,
                    delta_x: None,
                    delta_y: None,
                });
            }
        })
    };
    
    // マウスアップイベントハンドラー
    let on_mouse_up = {
        let on_mouse_event = props.on_mouse_event.clone();
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |e: MouseEvent| {
            if !props.connected {
                return;
            }
            
            e.prevent_default();
            
            // マウスボタンを判定
            let button = match e.button() {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                _ => MouseButton::Left,
            };
            
            // キャンバス内の座標を計算
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let rect = canvas.get_bounding_client_rect();
                let x = (e.client_x() as f64 - rect.left()) as i32;
                let y = (e.client_y() as f64 - rect.top()) as i32;
                
                // スケーリング計算
                let (scaled_x, scaled_y) = calculate_scaled_position(
                    x, y, canvas.width(), canvas.height(), props.width, props.height
                );
                
                on_mouse_event.emit(MouseEventInfo {
                    event_type: "up".to_string(),
                    button: Some(button),
                    x: scaled_x,
                    y: scaled_y,
                    delta_x: None,
                    delta_y: None,
                });
            }
        })
    };
    
    // マウス移動イベントハンドラー
    let on_mouse_move = {
        let on_mouse_event = props.on_mouse_event.clone();
        let canvas_ref = canvas_ref.clone();
        let last_mouse_pos = last_mouse_pos.clone();
        Callback::from(move |e: MouseEvent| {
            if !props.connected {
                return;
            }
            
            e.prevent_default();
            
            // キャンバス内の座標を計算
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let rect = canvas.get_bounding_client_rect();
                let x = (e.client_x() as f64 - rect.left()) as i32;
                let y = (e.client_y() as f64 - rect.top()) as i32;
                
                // 最後の位置と同じなら無視（パフォーマンス最適化）
                let (last_x, last_y) = *last_mouse_pos;
                if x == last_x && y == last_y {
                    return;
                }
                
                // 最後のマウス位置を更新
                last_mouse_pos.set((x, y));
                
                // スケーリング計算
                let (scaled_x, scaled_y) = calculate_scaled_position(
                    x, y, canvas.width(), canvas.height(), props.width, props.height
                );
                
                on_mouse_event.emit(MouseEventInfo {
                    event_type: "move".to_string(),
                    button: None,
                    x: scaled_x,
                    y: scaled_y,
                    delta_x: None,
                    delta_y: None,
                });
            }
        })
    };
    
    // マウスホイールイベントハンドラー
    let on_wheel = {
        let on_mouse_event = props.on_mouse_event.clone();
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |e: WheelEvent| {
            if !props.connected {
                return;
            }
            
            e.prevent_default();
            
            // デルタ値を取得（正規化）
            let delta_x = e.delta_x() as i32;
            let delta_y = e.delta_y() as i32;
            
            // キャンバス内の座標を計算
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let rect = canvas.get_bounding_client_rect();
                let x = (e.client_x() as f64 - rect.left()) as i32;
                let y = (e.client_y() as f64 - rect.top()) as i32;
                
                // スケーリング計算
                let (scaled_x, scaled_y) = calculate_scaled_position(
                    x, y, canvas.width(), canvas.height(), props.width, props.height
                );
                
                on_mouse_event.emit(MouseEventInfo {
                    event_type: "wheel".to_string(),
                    button: None,
                    x: scaled_x,
                    y: scaled_y,
                    delta_x: Some(delta_x),
                    delta_y: Some(delta_y),
                });
            }
        })
    };
    
    // コンテキストメニュー抑制ハンドラー
    let on_context_menu = Callback::from(|e: MouseEvent| {
        e.prevent_default();
    });
    
    // キーダウンイベントハンドラー
    let on_key_down = {
        let on_key_event = props.on_key_event.clone();
        Callback::from(move |e: KeyboardEvent| {
            if !props.connected {
                return;
            }
            
            // ブラウザのデフォルト動作を抑制（F5リロードなど）
            e.prevent_default();
            
            // 修飾キーの状態を取得
            let mut modifiers = Vec::new();
            if e.shift_key() {
                modifiers.push("Shift".to_string());
            }
            if e.ctrl_key() {
                modifiers.push("Control".to_string());
            }
            if e.alt_key() {
                modifiers.push("Alt".to_string());
            }
            if e.meta_key() {
                modifiers.push("Meta".to_string());
            }
            
            on_key_event.emit(KeyEventInfo {
                event_type: "down".to_string(),
                key_code: e.key(),
                modifiers,
            });
        })
    };
    
    // キーアップイベントハンドラー
    let on_key_up = {
        let on_key_event = props.on_key_event.clone();
        Callback::from(move |e: KeyboardEvent| {
            if !props.connected {
                return;
            }
            
            e.prevent_default();
            
            // 修飾キーの状態を取得
            let mut modifiers = Vec::new();
            if e.shift_key() {
                modifiers.push("Shift".to_string());
            }
            if e.ctrl_key() {
                modifiers.push("Control".to_string());
            }
            if e.alt_key() {
                modifiers.push("Alt".to_string());
            }
            if e.meta_key() {
                modifiers.push("Meta".to_string());
            }
            
            on_key_event.emit(KeyEventInfo {
                event_type: "up".to_string(),
                key_code: e.key(),
                modifiers,
            });
        })
    };
    
    // タッチスタートイベントハンドラー
    let on_touch_start = {
        let on_mouse_event = props.on_mouse_event.clone();
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |e: TouchEvent| {
            if !props.connected {
                return;
            }
            
            e.prevent_default();
            
            // タッチ数に応じたボタンを判定
            let button = match e.touches().dyn_into::<web_sys::TouchList>().ok().map(|list| list.length()) {
                1 => MouseButton::Left,
                2 => MouseButton::Right,
                _ => MouseButton::Left,
            };
            
            // 最初のタッチ位置を取得
            if let Some(touch_list) = e.touches().dyn_into::<web_sys::TouchList>().ok() {
                if let Some(touch) = touch_list.get(0) {
                    if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                        let rect = canvas.get_bounding_client_rect();
                        let x = (touch.client_x() as f64 - rect.left()) as i32;
                        let y = (touch.client_y() as f64 - rect.top()) as i32;
                        
                        // スケーリング計算
                        let (scaled_x, scaled_y) = calculate_scaled_position(
                            x, y, canvas.width(), canvas.height(), props.width, props.height
                        );
                        
                        on_mouse_event.emit(MouseEventInfo {
                            event_type: "down".to_string(),
                            button: Some(button),
                            x: scaled_x,
                            y: scaled_y,
                            delta_x: None,
                            delta_y: None,
                        });
                    }
                }
            }
        })
    };
    
    // タッチムーブイベントハンドラー
    let on_touch_move = {
        let on_mouse_event = props.on_mouse_event.clone();
        let canvas_ref = canvas_ref.clone();
        let last_mouse_pos = last_mouse_pos.clone();
        Callback::from(move |e: TouchEvent| {
            if !props.connected {
                return;
            }
            
            e.prevent_default();
            
            // 最初のタッチ位置を取得
            if let Some(touch_list) = e.touches().dyn_into::<web_sys::TouchList>().ok() {
                if let Some(touch) = touch_list.get(0) {
                    if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                        let rect = canvas.get_bounding_client_rect();
                        let x = (touch.client_x() as f64 - rect.left()) as i32;
                        let y = (touch.client_y() as f64 - rect.top()) as i32;
                        
                        // 最後の位置と同じなら無視（パフォーマンス最適化）
                        let (last_x, last_y) = *last_mouse_pos;
                        if x == last_x && y == last_y {
                            return;
                        }
                        
                        // 最後のマウス位置を更新
                        last_mouse_pos.set((x, y));
                        
                        // スケーリング計算
                        let (scaled_x, scaled_y) = calculate_scaled_position(
                            x, y, canvas.width(), canvas.height(), props.width, props.height
                        );
                        
                        on_mouse_event.emit(MouseEventInfo {
                            event_type: "move".to_string(),
                            button: None,
                            x: scaled_x,
                            y: scaled_y,
                            delta_x: None,
                            delta_y: None,
                        });
                    }
                }
            }
        })
    };
    
    // タッチエンドイベントハンドラー
    let on_touch_end = {
        let on_mouse_event = props.on_mouse_event.clone();
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |e: TouchEvent| {
            if !props.connected {
                return;
            }
            
            e.prevent_default();
            
            // タッチ数に応じたボタンを判定
            let button = match e.touches().length() {
                0 => MouseButton::Left, // すべてのタッチが終了した場合
                1 => MouseButton::Right, // 2本指から1本になった場合
                _ => MouseButton::Left,
            };
            
            // 最後のタッチ位置を取得（ない場合は最後のマウス位置を使用）
            let (x, y) = if let Some(touch_list) = e.changed_touches().dyn_into::<web_sys::TouchList>().ok() {
                if let Some(touch) = touch_list.get(0) {
                    if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                        let rect = canvas.get_bounding_client_rect();
                        ((touch.client_x() as f64 - rect.left()) as i32,
                         (touch.client_y() as f64 - rect.top()) as i32)
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };
            
            // スケーリング計算
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let (scaled_x, scaled_y) = calculate_scaled_position(
                    x, y, canvas.width(), canvas.height(), props.width, props.height
                );
                
                on_mouse_event.emit(MouseEventInfo {
                    event_type: "up".to_string(),
                    button: Some(button),
                    x: scaled_x,
                    y: scaled_y,
                    delta_x: None,
                    delta_y: None,
                });
            }
        })
    };
    
    // フォーカスハンドラー
    let on_focus = {
        let is_focused = is_focused.clone();
        Callback::from(move |_| {
            is_focused.set(true);
        })
    };
    
    // ブラーハンドラー
    let on_blur = {
        let is_focused = is_focused.clone();
        Callback::from(move |_| {
            is_focused.set(false);
        })
    };
    
    // キャンバスをクリックしたときのハンドラー
    let on_click = {
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                canvas.focus().unwrap_or_default();
            }
        })
    };
    
    // 画像を描画する副作用
    use_effect_with_deps(
        move |(image_data, canvas_ref)| {
            if let Some(image_data) = image_data {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    let ctx = canvas
                        .get_context("2d")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<web_sys::CanvasRenderingContext2d>()
                        .unwrap();
                    
                    // 画像を読み込んで描画
                    let image = web_sys::HtmlImageElement::new().unwrap();
                    let image_clone = image.clone();
                    let ctx_clone = ctx.clone();
                    let canvas_clone = canvas.clone();
                    
                    // 画像読み込み完了時のコールバック
                    let onload_callback = Closure::wrap(Box::new(move || {
                        // キャンバスをクリア
                        ctx_clone.clear_rect(0.0, 0.0, canvas_clone.width() as f64, canvas_clone.height() as f64);
                        
                        // 画像を描画
                        ctx_clone.draw_image_with_html_image_element(
                            &image_clone,
                            0.0,
                            0.0
                        ).unwrap_or_else(|_| {
                            // エラー処理
                        });
                    }) as Box<dyn FnMut()>);
                    
                    image.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
                    image.set_src(image_data);
                    
                    // Closureをリークしないように保持
                    onload_callback.forget();
                }
            }
            
            || () // クリーンアップ関数
        },
        (props.image_data.clone(), canvas_ref.clone()),
    );
    
    // キャンバスのサイズを設定する副作用
    use_effect_with_deps(
        move |(width, height, canvas_ref)| {
            if let (Some(w), Some(h)) = (*width, *height) {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    canvas.set_width(w);
                    canvas.set_height(h);
                }
            }
            
            || () // クリーンアップ関数
        },
        (props.width, props.height, canvas_ref.clone()),
    );
    
    html! {
        <div class="remote-display-container">
            <canvas
                ref={canvas_ref}
                class={classes!("remote-display", if *is_focused { "focused" } else { "" })}
                onmousedown={on_mouse_down}
                onmouseup={on_mouse_up}
                onmousemove={on_mouse_move}
                onwheel={on_wheel}
                oncontextmenu={on_context_menu}
                onclick={on_click}
                onkeydown={on_key_down}
                onkeyup={on_key_up}
                ontouchstart={on_touch_start}
                ontouchmove={on_touch_move}
                ontouchend={on_touch_end}
                onfocus={on_focus}
                onblur={on_blur}
                tabindex="0"
            >
                {
                    if !props.connected {
                        html! {
                            <div class="overlay">
                                <div class="message">{"接続されていません"}</div>
                            </div>
                        }
                    } else if props.image_data.is_none() {
                        html! {
                            <div class="overlay">
                                <div class="message">{"画面データを読み込み中..."}</div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </canvas>
        </div>
    }
}

/// キャンバス座標をリモート画面座標に変換する
fn calculate_scaled_position(
    x: i32, 
    y: i32, 
    canvas_width: u32, 
    canvas_height: u32,
    remote_width: Option<u32>,
    remote_height: Option<u32>
) -> (i32, i32) {
    // リモート画面サイズが指定されていない場合はそのまま返す
    if remote_width.is_none() || remote_height.is_none() {
        return (x, y);
    }
    
    let remote_width = remote_width.unwrap();
    let remote_height = remote_height.unwrap();
    
    // スケール比を計算
    let scale_x = remote_width as f64 / canvas_width as f64;
    let scale_y = remote_height as f64 / canvas_height as f64;
    
    // 座標を変換
    let scaled_x = (x as f64 * scale_x) as i32;
    let scaled_y = (y as f64 * scale_y) as i32;
    
    (scaled_x, scaled_y)
}