# リモートデスクトップアプリケーション(未完成)

## 概要

このプロジェクトは、Rustで開発されたクロスプラットフォームのリモートデスクトップアプリケーションです。Windows、macOS、Linuxに対応しており、デスクトップクライアント、Webクライアント、サーバーコンポーネントから構成されています。

## 主な機能

- マルチプラットフォーム対応（Windows、macOS、Linux）
- デスクトップクライアントとWebクライアント
- 複数の接続方式（WebSocket、WebRTC、TCP）
- 画面共有と遠隔操作
- ファイル転送
- クリップボード共有
- システム情報のモニタリング
- マルチディスプレイサポート
- 画像圧縮（WebP、JPEG、PNG）
- エンドツーエンド暗号化（AES-GCM、ChaCha20Poly1305）

## システム要件

### サーバー
- Windows 10/11、macOS 10.15以降、または最新のLinuxディストリビューション
- 画面共有のための十分なCPUリソース
- 最低8GBのRAM

### クライアント
- デスクトップクライアント: Windows 10/11、macOS 10.15以降、または最新のLinuxディストリビューション
- Webクライアント: 最新のChrome、Firefox、Edge、Safariブラウザ

## プロジェクト構造

```
remote-desktop/
├── common/                      # 共通ライブラリ
│   ├── src/
│   │   ├── crypto/              # 暗号化モジュール
│   │   ├── compression/         # 画像圧縮モジュール
│   │   ├── protocol/            # プロトコル定義
│   │   └── utils/               # ユーティリティ関数
│   └── Cargo.toml
│
├── server/                      # サーバー実装
│   ├── src/
│   │   ├── capture/             # 画面キャプチャモジュール
│   │   ├── input/               # 入力ハンドリング
│   │   ├── network/             # ネットワーク管理
│   │   ├── auth/                # 認証
│   │   └── main.rs              # サーバーエントリーポイント
│   └── Cargo.toml
│
├── client/                      # デスクトップクライアント
│   ├── src/
│   │   ├── ui/                  # eframe/eguiベースのUI
│   │   ├── network/             # サーバー接続
│   │   ├── input/               # リモート入力送信
│   │   └── main.rs              # クライアントエントリーポイント
│   └── Cargo.toml
│
├── web-client/                  # Webクライアント
│   ├── src/                     # Rust/Yew/WebAssembly
│   ├── public/                  # 静的アセット
│   ├── package.json             # NPM設定
│   └── webpack.config.js        # Webpack設定
│
├── docs/                        # ドキュメント
│   ├── protocol.md              # プロトコル仕様
│   └── api.md                   # API仕様
│
├── tests/                       # 統合テスト
│
├── Cargo.toml                   # ワークスペース設定
└── README.md                    # このファイル
```

## インストール方法

### ソースからビルド

1. Rustツールチェーンをインストール
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. リポジトリをクローン
```bash
git clone https://github.com/yourusername/remote-desktop-rs.git
cd remote-desktop-rs
```

3. サーバーをビルド
```bash
cd server
cargo build --release
```

4. デスクトップクライアントをビルド
```bash
cd ../client
cargo build --release
```

5. Webクライアントをビルド
```bash
cd ../web-client
npm install
npm run build
```

## 使用方法

### サーバーの起動

```bash
cd server
cargo run --release
```

サーバーはデフォルトで9090ポートをリッスンします。

### デスクトップクライアントの使用

```bash
cd client
cargo run --release
```

起動後、サーバーのIPアドレスとポートを入力して接続します。

### Webクライアントの使用

開発環境での実行:
```bash
cd web-client
npm run start
```

ブラウザで http://localhost:8080 を開き、サーバーのIPアドレスとポートを入力して接続します。

## 開発

### 依存関係

- Rust 1.68.0以降
- Node.js 16.x以降（Webクライアント用）
- 画像処理ライブラリ（WebP、JPEG、PNGサポート用）

### 機能フラグ

サーバーとクライアントはさまざまな機能フラグを持っています:

```bash
# サーバー（すべての機能を有効化）
cargo build --release --features full

# クライアント（WebP圧縮サポートを有効化）
cargo build --release --features webp-support
```

## ライセンス

MIT

## 貢献

エラー修正やバグ報告、機能リクエストは、GitHubのIssuesにて受け付けています。プルリクエストも歓迎します。

## 謝辞

このプロジェクトは以下のオープンソースライブラリに依存しています:

- Rust、WebAssembly
- Yew、egui、eframe
- tokio、tungstenite
- serde、image、webp
- その他多くのRustエコシステムライブラリ
