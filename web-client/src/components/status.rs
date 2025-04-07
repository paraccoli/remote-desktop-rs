//! ステータスバーコンポーネント
//!
//! アプリケーションのステータス情報を表示するUIコンポーネントです。

use wasm_bindgen::prelude::*;
use yew::prelude::*;

/// パフォーマンス情報
#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceInfo {
    /// FPS
    pub fps: f32,
    /// レイテンシー (ms)
    pub latency: u64,
    /// 受信データ量 (バイト)
    pub received_bytes: u64,
    /// 送信データ量 (バイト)
    pub sent_bytes: u64,
    /// 画質
    pub quality: u8,
}

impl Default for PerformanceInfo {
    fn default() -> Self {
        Self {
            fps: 0.0,
            latency: 0,
            received_bytes: 0,
            sent_bytes: 0,
            quality: 0,
        }
    }
}

/// システム情報
#[derive(Clone, Debug, PartialEq)]
pub struct SystemInfo {
    /// CPUモデル
    pub cpu_model: String,
    /// CPU使用率
    pub cpu_usage: f32,
    /// メモリ合計（バイト）
    pub total_memory: u64,
    /// メモリ使用量（バイト）
    pub used_memory: u64,
    /// OSバージョン
    pub os_version: String,
    /// ホスト名
    pub hostname: String,
    /// 稼働時間（秒）
    pub uptime: u64,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            cpu_model: String::from("不明"),
            cpu_usage: 0.0,
            total_memory: 0,
            used_memory: 0,
            os_version: String::from("不明"),
            hostname: String::from("不明"),
            uptime: 0,
        }
    }
}

/// 接続情報
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionStatus {
    /// 接続状態
    pub connected: bool,
    /// 接続タイプ (WebSocket, WebRTC)
    pub connection_type: String,
    /// サーバーアドレス
    pub server_address: String,
    /// TLS使用フラグ
    pub using_tls: bool,
    /// 接続時間（秒）
    pub connection_time: u64,
    /// ステータスメッセージ
    pub status_message: String,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self {
            connected: false,
            connection_type: String::from("なし"),
            server_address: String::from(""),
            using_tls: false,
            connection_time: 0,
            status_message: String::from("切断"),
        }
    }
}

/// ステータスバーのプロパティ
#[derive(Properties, Clone, PartialEq)]
pub struct StatusBarProps {
    /// パフォーマンス情報
    pub performance: PerformanceInfo,
    /// システム情報
    pub system_info: Option<SystemInfo>,
    /// 接続情報
    pub connection: ConnectionStatus,
    /// 詳細表示フラグ
    pub show_details: bool,
    /// 詳細表示切り替えハンドラー
    pub on_toggle_details: Callback<()>,
}

/// ステータスバーコンポーネント
#[function_component(StatusBar)]
pub fn status_bar(props: &StatusBarProps) -> Html {
    // 詳細表示切り替えハンドラー
    let on_toggle_details = {
        let on_toggle_details = props.on_toggle_details.clone();
        Callback::from(move |_| {
            on_toggle_details.emit(());
        })
    };

    // データサイズをフォーマット
    let format_bytes = |bytes: u64| -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    };

    // 稼働時間をフォーマット
    let format_uptime = |seconds: u64| -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if days > 0 {
            format!("{}日 {}時間 {}分 {}秒", days, hours, minutes, secs)
        } else if hours > 0 {
            format!("{}時間 {}分 {}秒", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}分 {}秒", minutes, secs)
        } else {
            format!("{}秒", secs)
        }
    };

    html! {
        <div class="status-bar">
            <div class="status-basic">
                <div class={classes!("connection-status", if props.connection.connected { "connected" } else { "disconnected" })}>
                    {
                        if props.connection.connected {
                            format!("接続中: {}", props.connection.server_address)
                        } else {
                            "切断".to_string()
                        }
                    }
                </div>
                <div class="performance-metrics">
                    <span class="fps-metric" title="フレームレート">
                        <i class="icon-display"></i>
                        {format!("{:.1} FPS", props.performance.fps)}
                    </span>
                    <span class="latency-metric" title="レイテンシー">
                        <i class="icon-clock"></i>
                        {format!("{} ms", props.performance.latency)}
                    </span>
                    <span class="quality-metric" title="画質">
                        <i class="icon-image"></i>
                        {format!("画質: {}%", props.performance.quality)}
                    </span>
                </div>
                <button 
                    class="details-toggle" 
                    onclick={on_toggle_details}
                    title={if props.show_details { "詳細を隠す" } else { "詳細を表示" }}
                >
                    {
                        if props.show_details {
                            "△"
                        } else {
                            "▽"
                        }
                    }
                </button>
            </div>

            {
                if props.show_details {
                    html! {
                        <div class="status-details">
                            <div class="details-section">
                                <h3>{"接続情報"}</h3>
                                <div class="detail-row">
                                    <span class="detail-label">{"接続タイプ:"}</span>
                                    <span class="detail-value">{&props.connection.connection_type}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="detail-label">{"サーバー:"}</span>
                                    <span class="detail-value">{&props.connection.server_address}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="detail-label">{"暗号化:"}</span>
                                    <span class="detail-value">{if props.connection.using_tls { "有効" } else { "無効" }}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="detail-label">{"接続時間:"}</span>
                                    <span class="detail-value">{format_uptime(props.connection.connection_time)}</span>
                                </div>
                            </div>

                            <div class="details-section">
                                <h3>{"パフォーマンス"}</h3>
                                <div class="detail-row">
                                    <span class="detail-label">{"FPS:"}</span>
                                    <span class="detail-value">{format!("{:.1}", props.performance.fps)}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="detail-label">{"レイテンシー:"}</span>
                                    <span class="detail-value">{format!("{} ms", props.performance.latency)}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="detail-label">{"受信データ:"}</span>
                                    <span class="detail-value">{format_bytes(props.performance.received_bytes)}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="detail-label">{"送信データ:"}</span>
                                    <span class="detail-value">{format_bytes(props.performance.sent_bytes)}</span>
                                </div>
                            </div>

                            {
                                if let Some(system_info) = &props.system_info {
                                    html! {
                                        <div class="details-section">
                                            <h3>{"システム情報"}</h3>
                                            <div class="detail-row">
                                                <span class="detail-label">{"CPU:"}</span>
                                                <span class="detail-value">{&system_info.cpu_model}</span>
                                            </div>
                                            <div class="detail-row">
                                                <span class="detail-label">{"CPU使用率:"}</span>
                                                <span class="detail-value">{format!("{:.1}%", system_info.cpu_usage)}</span>
                                            </div>
                                            <div class="detail-row">
                                                <span class="detail-label">{"メモリ:"}</span>
                                                <span class="detail-value">
                                                    {format!("{} / {}", 
                                                        format_bytes(system_info.used_memory), 
                                                        format_bytes(system_info.total_memory))}
                                                </span>
                                            </div>
                                            <div class="detail-row">
                                                <span class="detail-label">{"OS:"}</span>
                                                <span class="detail-value">{&system_info.os_version}</span>
                                            </div>
                                            <div class="detail-row">
                                                <span class="detail-label">{"ホスト名:"}</span>
                                                <span class="detail-value">{&system_info.hostname}</span>
                                            </div>
                                            <div class="detail-row">
                                                <span class="detail-label">{"稼働時間:"}</span>
                                                <span class="detail-value">{format_uptime(system_info.uptime)}</span>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}