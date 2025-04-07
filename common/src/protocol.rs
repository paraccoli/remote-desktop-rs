//! 通信プロトコル定義
//!
//! リモートデスクトップアプリケーションで使用される通信プロトコルを定義します。
//! クライアントとサーバー間で送受信されるコマンドとレスポンスを含みます。

use serde::{Serialize, Deserialize};
use std::time::Duration;

/// マウスボタン
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    /// 左ボタン
    Left,
    /// 右ボタン
    Right,
    /// 中ボタン
    Middle,
    /// サイドボタン（戻る）
    Back,
    /// サイドボタン（進む）
    Forward,
}

/// キー修飾子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyModifier {
    /// Shiftキー
    Shift,
    /// Controlキー
    Control,
    /// Altキー
    Alt,
    /// Commandキー（Mac）/ Windowsキー（Windows）
    Meta,
    /// CapsLockキー
    CapsLock,
    /// NumLockキー
    NumLock,
}

/// 画像形式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    /// JPEG
    JPEG,
    /// PNG
    PNG,
    /// WebP
    WebP,
    /// AVIF
    AVIF,
}

/// 画質設定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    /// 最高品質（ロスレス圧縮）
    Best,
    /// 高品質
    High,
    /// 中品質
    Medium,
    /// 低品質
    Low,
    /// カスタム品質
    Custom(u8),
}

impl QualityPreset {
    /// 品質値（0-100）を取得
    pub fn quality_value(&self) -> u8 {
        match self {
            QualityPreset::Best => 100,
            QualityPreset::High => 85,
            QualityPreset::Medium => 70,
            QualityPreset::Low => 50,
            QualityPreset::Custom(value) => *value,
        }
    }
    
    /// 推奨画像フォーマットを取得
    pub fn recommended_format(&self) -> ImageFormat {
        match self {
            QualityPreset::Best => ImageFormat::PNG,
            QualityPreset::High => ImageFormat::WebP,
            QualityPreset::Medium => ImageFormat::WebP,
            QualityPreset::Low => ImageFormat::JPEG,
            QualityPreset::Custom(value) if *value > 90 => ImageFormat::PNG,
            QualityPreset::Custom(_) => ImageFormat::WebP,
        }
    }
}

/// 接続状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// 未接続
    Disconnected,
    /// 接続中
    Connecting,
    /// 接続済み
    Connected,
    /// 認証中
    Authenticating,
    /// エラー
    Error,
}

/// クライアントからサーバーへのコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// 認証
    Authenticate {
        /// ユーザー名
        username: String,
        /// パスワード（ハッシュ済み）
        password_hash: String,
        /// クライアント情報
        client_info: ClientInfo,
    },
    
    /// スクリーンショット要求
    RequestScreenshot {
        /// 画質（1-100）
        quality: Option<u8>,
        /// 幅（オプション）
        width: Option<u32>,
        /// 高さ（オプション）
        height: Option<u32>,
        /// モニターインデックス（オプション）
        monitor: Option<usize>,
    },
    
    /// マウス移動
    MouseMove {
        /// X座標
        x: i32,
        /// Y座標
        y: i32,
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
        delta_x: i32,
        /// Y軸スクロール量
        delta_y: i32,
    },
    
    /// キー押下
    KeyDown {
        /// キーコード
        key_code: u32,
        /// 修飾キー
        modifiers: Vec<KeyModifier>,
    },
    
    /// キー解放
    KeyUp {
        /// キーコード
        key_code: u32,
        /// 修飾キー
        modifiers: Vec<KeyModifier>,
    },
    
    /// テキスト入力
    TextInput {
        /// 入力テキスト
        text: String,
    },
    
    /// キーコンビネーション
    KeyCombo {
        /// キーコードのリスト
        key_codes: Vec<u32>,
        /// 修飾キー
        modifiers: Vec<KeyModifier>,
    },
    
    /// 画質設定
    SetQuality {
        /// 画質値（1-100）
        quality: u8,
    },
    
    /// 画像形式設定
    SetImageFormat {
        /// 画像形式
        format: ImageFormat,
    },
    
    /// FPS設定
    SetFps {
        /// フレームレート
        fps: u8,
    },
    
    /// アプリケーション実行
    RunApplication {
        /// 実行コマンド
        command: String,
    },
    
    /// システム情報要求
    RequestSystemInfo,
    
    /// クリップボード取得要求
    RequestClipboardContent,
    
    /// クリップボード設定
    SetClipboardContent {
        /// クリップボード内容
        content: String,
    },
    
    /// ファイル転送開始
    StartFileTransfer {
        /// ファイル名
        filename: String,
        /// ファイルサイズ
        size: u64,
        /// チェックサム
        checksum: String,
    },
    
    /// ファイルデータ
    FileData {
        /// 転送ID
        transfer_id: u32,
        /// データ
        data: Vec<u8>,
        /// オフセット
        offset: u64,
    },
    
    /// Ping
    Ping {
        /// タイムスタンプ（ミリ秒）
        timestamp: u64,
    },
    
    /// 切断
    Disconnect,
}

/// サーバーからクライアントへのレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// 認証結果
    AuthResult {
        /// 成功したかどうか
        success: bool,
        /// メッセージ
        message: String,
    },
    
    /// スクリーンショットデータ
    ScreenshotData {
        /// 画像データ
        data: Vec<u8>,
        /// 画像形式
        format: ImageFormat,
        /// 幅
        width: u32,
        /// 高さ
        height: u32,
        /// タイムスタンプ（ミリ秒）
        timestamp: u64,
    },
    
    /// コマンド実行結果
    CommandResult {
        /// 成功したかどうか
        success: bool,
        /// メッセージ
        message: String,
        /// コマンド固有のデータ
        data: Option<serde_json::Value>,
    },
    
    /// システム情報
    SystemInfo {
        /// CPUモデル
        cpu_model: String,
        /// CPU使用率
        cpu_usage: f32,
        /// メモリ合計（バイト）
        total_memory: u64,
        /// メモリ使用量（バイト）
        used_memory: u64,
        /// OSバージョン
        os_version: String,
        /// ホスト名
        hostname: String,
        /// 稼働時間（秒）
        uptime: u64,
    },
    
    /// クリップボード内容
    ClipboardContent {
        /// クリップボード内容
        content: String,
    },
    
    /// ファイル転送状態
    FileTransferStatus {
        /// 転送ID
        transfer_id: u32,
        /// 成功したかどうか
        success: bool,
        /// メッセージ
        message: String,
        /// 現在の進捗（バイト）
        progress: u64,
        /// 合計サイズ（バイト）
        total_size: u64,
    },
    
    /// 接続状態
    ConnectionStatus {
        /// 接続されているかどうか
        connected: bool,
        /// メッセージ
        message: String,
    },
    
    /// Pong
    Pong {
        /// 元のタイムスタンプ
        original_timestamp: u64,
        /// サーバー時間
        server_time: u64,
    },
    
    /// エラー
    Error {
        /// エラーコード
        code: i32,
        /// エラーメッセージ
        message: String,
    },
}

/// クライアント情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// クライアントアプリケーション名
    pub app_name: String,
    /// クライアントバージョン
    pub version: String,
    /// OSタイプ
    pub os_type: String,
    /// OSバージョン
    pub os_version: String,
    /// デバイス名
    pub device_name: String,
    /// 画面解像度（幅）
    pub screen_width: u32,
    /// 画面解像度（高さ）
    pub screen_height: u32,
    /// クライアントが対応する機能フラグ
    pub capabilities: Vec<String>,
}

/// 接続情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// ホスト名または IP アドレス
    pub host: String,
    /// ポート番号
    pub port: u16,
    /// 接続プロトコル（tcp, websocket, webrtc）
    pub protocol: String,
    /// ユーザー名（オプション）
    pub username: Option<String>,
    /// パスワード（オプション）
    pub password: Option<String>,
    /// 接続タイムアウト（ミリ秒）
    pub timeout_ms: u64,
    /// TLS を使用するかどうか
    pub use_tls: bool,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9999,
            protocol: "tcp".to_string(),
            username: None,
            password: None,
            timeout_ms: 5000,
            use_tls: false,
        }
    }
}

impl ConnectionInfo {
    /// デバッグ表示用に文字列化（パスワードは隠す）
    pub fn to_debug_string(&self) -> String {
        let protocol_prefix = if self.use_tls { 
            match self.protocol.as_str() {
                "tcp" => "tls",
                "websocket" => "wss",
                _ => &self.protocol,
            }
        } else {
            &self.protocol
        };
        
        let auth_part = if let Some(username) = &self.username {
            format!("{}:***@", username)
        } else {
            String::new()
        };
        
        format!("{}://{}{}:{}", protocol_prefix, auth_part, self.host, self.port)
    }
    
    /// タイムアウトを設定
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_ms = timeout.as_millis() as u64;
        self
    }
    
    /// TLSを設定
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }
    
    /// 認証情報を設定
    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }
}