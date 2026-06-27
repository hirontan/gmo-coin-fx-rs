# gmo-coin-fx-rs

GMO コイン FX API の Rust クライアントライブラリです。REST API および WebSocket API に対応しています。

[![CI](https://github.com/hirontan/gmo-coin-fx-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/hirontan/gmo-coin-fx-rs/actions/workflows/ci.yml)

## クレート構成

| クレート | 説明 |
|---|---|
| `gmo-coin-fx-core` | ドメインモデル・認証 (HMAC-SHA256) |
| `gmo-coin-fx-client` | 非同期 REST API クライアント |
| `gmo-coin-fx-ws` | WebSocket クライアント（自動再接続・型安全デシリアライズ） |
| `gmo-coin-fx-domain-risk` | ポジションサイズ・証拠金・レバレッジ・注文前リスク判定 |

## インストール

`Cargo.toml` に以下を追加してください。`tag` には最新のリリースタグを指定します。

### REST API クライアント

```toml
[dependencies]
gmo-coin-fx-client = { git = "https://github.com/hirontan/gmo-coin-fx-rs.git", tag = "v0.1.0" }
```

### WebSocket クライアント

```toml
[dependencies]
gmo-coin-fx-ws = { git = "https://github.com/hirontan/gmo-coin-fx-rs.git", tag = "v0.1.0" }
```

### コアモデルのみ

```toml
[dependencies]
gmo-coin-fx-core = { git = "https://github.com/hirontan/gmo-coin-fx-rs.git", tag = "v0.1.0" }
```

## 使い方

### パブリック REST API

```rust
use gmo_coin_fx_client::GmoFxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GmoFxClient::builder().build();

    let tickers = client.ticker().await?;
    println!("{:?}", tickers);

    Ok(())
}
```

### プライベート REST API

```rust
use gmo_coin_fx_client::GmoFxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GmoFxClient::builder()
        .credentials("YOUR_API_KEY", "YOUR_SECRET_KEY")
        .build();

    let assets = client.assets().await?;
    println!("{:?}", assets);

    Ok(())
}
```

### WebSocket（パブリック）

```rust
use gmo_coin_fx_core::models::{Channel, FxSymbol, Subscription};
use gmo_coin_fx_core::models::ws_events::PublicWsMessage;
use gmo_coin_fx_ws::PublicWsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PublicWsClient::connect().await?;
    let sub = Subscription::builder()
        .channel(Channel::Ticker)
        .symbol(FxSymbol::UsdJpy)
        .build();
    client.subscribe(sub).await?;

    while let Some(msg) = client.next_message().await? {
        match msg {
            PublicWsMessage::Ticker(t) => {
                println!("Ask: {}, Bid: {}", t.ask, t.bid);
            }
        }
    }

    Ok(())
}
```

### WebSocket（プライベート）

```rust
use gmo_coin_fx_client::GmoFxClient;
use gmo_coin_fx_core::models::{Channel, Subscription};
use gmo_coin_fx_core::models::ws_events::PrivateWsMessage;
use gmo_coin_fx_ws::PrivateWsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rest_client = GmoFxClient::builder()
        .credentials("YOUR_API_KEY", "YOUR_SECRET_KEY")
        .build();

    // トークン取得・自動更新付きで接続
    let mut ws_client = PrivateWsClient::connect(rest_client).await?;
    let sub = Subscription::builder()
        .channel(Channel::ExecutionEvents)
        .build();
    ws_client.subscribe(sub).await?;

    while let Some(msg) = ws_client.next_message().await? {
        match msg {
            PrivateWsMessage::Execution(e) => {
                println!("約定: {} {} {}", e.symbol, e.side, e.execution_size);
            }
            PrivateWsMessage::Position(p) => {
                println!("建玉更新: {:?}", p);
            }
            PrivateWsMessage::Order(o) => {
                println!("注文更新: {:?}", o);
            }
        }
    }

    Ok(())
}
```

## 機能

- ✅ Public REST API: ステータス・ティッカー・銘柄一覧・ローソク足
- ✅ Private REST API: 資産・注文・アクティブ注文・約定・建玉・キャンセル・決済・スピード注文
- ✅ Public WebSocket: ティッカー（自動再接続・購読復元）
- ✅ Private WebSocket: 約定・建玉・注文イベント（自動再接続・トークン自動更新）

## FXリスク計算

レバレッジ、必要証拠金、証拠金維持率などの計算式や、注文前リスクチェック機能（`RiskGuard`）については、[FXリスク計算ドキュメント](docs/risk-calculation.md) を参照してください。

## ライセンス

MIT OR Apache-2.0

---

## 開発者向け（batonel）

```shell
curl -fsSL https://raw.githubusercontent.com/Arcflect/batonel/main/scripts/install-batonel.sh | bash -s -- v1.13.0
batonel scaffold
batonel plan
batonel verify
```

### コミットメッセージの制約

本プロジェクトでは、コミットログの一貫性を保つため、以下の Conventional Commits 規則に準拠したコミットメッセージと細かい粒度でのコミットを制約として設けています。

#### メッセージフォーマット
```
<type>(<scope>): <description>
```
※ `<scope>` は省略可能です。

#### 主要な種類（`<type>`）
- `feat`: 新機能の追加
- `fix`: バグの修正
- `docs`: ドキュメントのみの変更（READMEや各種設計書など）
- `style`: コードの動作に影響を与えない修正（フォーマット調整、スペース変更、セミコロン修正など）
- `refactor`: 新機能の追加やバグの修正を行わない、コードのリファクタリング
- `perf`: パフォーマンス向上のためのコード変更
- `test`: テストコードの追加・修正
- `chore`: ビルドツール、パッケージマネージャー、依存ライブラリの更新など

#### コミットの粒度
- 大きな変更を一括でコミットするのではなく、機能や変更ごとに細分化して細かく（Granular）コミットを積み重ねるようにしてください。
