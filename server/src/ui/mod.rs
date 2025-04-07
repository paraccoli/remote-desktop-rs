//! UIモジュール
//!
//! このモジュールはリモートデスクトップサーバーのユーザーインターフェースを担当します。

pub mod settings;

pub use settings::ServerSettings;

/// サーバー状態
#[derive(Debug, Clone)]
pub struct ServerState {
    /// サーバーが実行中かどうか
    pub running: bool,
    /// 接続クライアント数
    pub connected_clients: usize,
    /// サーバーアドレス
    pub server_address: String,
    /// サーバーポート
    pub server_port: u16,
    /// TLS有効フラグ
    pub tls_enabled: bool,
    /// 現在のセッション情報
    pub sessions: Vec<SessionStatus>,
}

/// セッションステータス
#[derive(Debug, Clone)]
pub struct SessionStatus {
    /// セッションID
    pub id: String,
    /// クライアントIPアドレス
    pub ip_address: String,
    /// 認証済みフラグ
    pub authenticated: bool,
    /// 接続時間（秒）
    pub connection_time: u64,
    /// アイドル時間（秒）
    pub idle_time: u64,
    /// クライアント情報
    pub client_info: Option<String>,
    /// レイテンシー（ミリ秒）
    pub latency: Option<u64>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            running: false,
            connected_clients: 0,
            server_address: "0.0.0.0".to_string(),
            server_port: 9090,
            tls_enabled: false,
            sessions: Vec::new(),
        }
    }
}

/// ステータス情報
#[derive(Debug, Clone)]
pub struct StatusInfo {
    /// CPU使用率
    pub cpu_usage: f32,
    /// メモリ使用量（MB）
    pub memory_usage: u64,
    /// 実行時間（秒）
    pub uptime: u64,
    /// 送信データ量（バイト）
    pub sent_bytes: u64,
    /// 受信データ量（バイト）
    pub received_bytes: u64,
}

impl Default for StatusInfo {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            uptime: 0,
            sent_bytes: 0,
            received_bytes: 0,
        }
    }
}