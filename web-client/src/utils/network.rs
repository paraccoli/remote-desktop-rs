//! ネットワークユーティリティ
//!
//! WebSocketやWebRTC通信に関するユーティリティ関数を提供します。

use wasm_bindgen::prelude::*;
use web_sys::{
    WebSocket, RtcPeerConnection, RtcConfiguration, RtcDataChannel,
    RtcSdpType, RtcSessionDescriptionInit, RtcIceCandidate, RtcIceCandidateInit
};
use js_sys::{Array, JSON, Object, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use std::fmt::Debug;

// RtcDataChannelのstateチェック関数の実装
use wasm_bindgen::JsValue;

// チャンネルの状態を取得する関数を作成
pub fn get_data_channel_state(channel: &web_sys::RtcDataChannel) -> u16 {
    js_sys::Reflect::get(channel, &JsValue::from_str("readyState"))
        .map(|val| val.as_f64().unwrap_or(0.0) as u16)
        .unwrap_or(0)
}

// 定数定義
pub const RTC_DATA_CHANNEL_CONNECTING: u16 = 0;
pub const RTC_DATA_CHANNEL_OPEN: u16 = 1;
pub const RTC_DATA_CHANNEL_CLOSING: u16 = 2;
pub const RTC_DATA_CHANNEL_CLOSED: u16 = 3;

// 状態を文字列で取得
pub fn get_data_channel_state_string(channel: &web_sys::RtcDataChannel) -> String {
    match get_data_channel_state(channel) {
        RTC_DATA_CHANNEL_CONNECTING => "接続中".to_string(),
        RTC_DATA_CHANNEL_OPEN => "接続済み".to_string(),
        RTC_DATA_CHANNEL_CLOSING => "切断中".to_string(),
        RTC_DATA_CHANNEL_CLOSED => "切断済み".to_string(),
        _ => "不明".to_string(),
    }
}

/// WebRTCがサポートされているかチェック
pub fn is_webrtc_supported() -> bool {
    js_sys::eval("typeof RTCPeerConnection !== 'undefined'")
        .map(|val| !val.is_undefined() && val.as_bool().unwrap_or(false))
        .unwrap_or(false)
}

/// WebSocket URLを構築
pub fn build_websocket_url(host: &str, port: u16, path: &str, use_tls: bool) -> String {
    let protocol = if use_tls { "wss" } else { "ws" };
    format!("{}://{}:{}{}", protocol, host, port, path)
}

/// HTTP URLを構築
pub fn build_http_url(host: &str, port: u16, path: &str, use_tls: bool) -> String {
    let protocol = if use_tls { "https" } else { "http" };
    format!("{}://{}:{}{}", protocol, host, port, path)
}

/// WebRTC接続に使用するICEサーバー設定を作成
pub fn create_ice_server_config() -> RtcConfiguration {
    let ice_servers = Array::new();
    
    // Googleの公開STUNサーバー
    let stun_server = Object::new();
    Reflect::set(&stun_server, &JsValue::from_str("urls"), &JsValue::from_str("stun:stun.l.google.com:19302")).unwrap();
    ice_servers.push(&stun_server);
    
    // WebRTC設定オブジェクトを作成
    let mut rtc_config = RtcConfiguration::new();
    rtc_config.set_ice_servers(&ice_servers);
    
    rtc_config
}

/// WebRTCのSDP提案を作成
pub async fn create_webrtc_offer(peer_connection: &RtcPeerConnection) -> Result<String, JsValue> {
    let offer = JsFuture::from(peer_connection.create_offer()).await?;
    let offer_sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))?
        .as_string()
        .ok_or_else(|| JsValue::from_str("SDP文字列の取得に失敗"))?;
    
    let mut desc_init = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    desc_init.set_sdp(&offer_sdp);
    
    JsFuture::from(peer_connection.set_local_description(&desc_init)).await?;
    
    Ok(offer_sdp)
}

/// WebRTCのSDP回答を処理
pub async fn process_webrtc_answer(peer_connection: &RtcPeerConnection, sdp: &str) -> Result<(), JsValue> {
    let mut desc_init = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    desc_init.set_sdp(sdp);
    
    JsFuture::from(peer_connection.set_remote_description(&desc_init)).await?;
    
    Ok(())
}

/// WebRTCのICE候補を追加
pub async fn add_ice_candidate(peer_connection: &RtcPeerConnection, candidate: &str, sdp_mid: &str, sdp_m_line_index: u16) -> Result<(), JsValue> {
    let mut candidate_init = RtcIceCandidateInit::new(candidate);
    candidate_init.set_sdp_mid(Some(sdp_mid));
    candidate_init.set_sdp_m_line_index(Some(sdp_m_line_index));
    
    let ice_candidate = RtcIceCandidate::new(&candidate_init)?;
    
    JsFuture::from(peer_connection.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&ice_candidate))).await?;
    
    Ok(())
}

/// WebSocketの接続状態を文字列に変換
pub fn websocket_state_to_string(ws: &WebSocket) -> String {
    match ws.ready_state() {
        WebSocket::CONNECTING => "接続中".to_string(),
        WebSocket::OPEN => "接続済み".to_string(),
        WebSocket::CLOSING => "切断中".to_string(),
        WebSocket::CLOSED => "切断済み".to_string(),
        _ => "不明".to_string(),
    }
}

/// WebRTCデータチャネルの状態を文字列に変換
pub fn data_channel_state_to_string(channel: &RtcDataChannel) -> String {
    match channel.ready_state() {
        RtcDataChannel::CONNECTING => "接続中".to_string(),
        RtcDataChannel::OPEN => "接続済み".to_string(),
        RtcDataChannel::CLOSING => "切断中".to_string(),
        RtcDataChannel::CLOSED => "切断済み".to_string(),
        _ => "不明".to_string(),
    }
}

/// レイテンシーを計算（送信時刻と現在時刻の差の半分）
pub fn calculate_latency(sent_timestamp: f64) -> u32 {
    let now = js_sys::Date::now();
    let round_trip_time = now - sent_timestamp;
    (round_trip_time / 2.0) as u32
}

/// FPSを計算（前回のフレーム時間と現在時間の差から）
pub fn calculate_fps(last_frame_time: f64) -> f32 {
    let now = js_sys::Date::now();
    let delta = now - last_frame_time;
    
    if delta > 0.0 {
        (1000.0 / delta) as f32
    } else {
        0.0
    }
}