//! Webクライアントのコンポーネントモジュール
//!
//! このモジュールには、Webクライアントで使用するReact/Wasmコンポーネントが含まれています。

pub mod connection;
pub mod controls;
pub mod display;
pub mod settings;
pub mod status;

// 主要コンポーネントをre-export
pub use connection::ConnectionForm;
pub use controls::ControlPanel;
pub use display::RemoteDisplay;
pub use settings::SettingsPanel;
pub use status::StatusBar;