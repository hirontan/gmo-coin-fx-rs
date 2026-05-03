//! # gmo-coin-fx-ws
//!
//! GMO コイン FX の WebSocket API クライアントクレートです。
//! パブリック（認証不要）とプライベート（認証必要）の両チャンネルに対応します。
//!
//! - **自動再接続**: ネットワーク切断時に Exponential Backoff で再接続を試みます。
//! - **購読の復元**: 再接続成功後に以前の購読を自動で再送信します。
//! - **型安全なデシリアライズ**: 受信 JSON を [`PublicWsMessage`] / [`PrivateWsMessage`] Enum に変換します。
//! - **トークン自動更新**: [`PrivateWsClient`] はバックグラウンドで 30 分ごとに認証トークンを延長します。

pub mod private;
pub mod public;

pub use private::PrivateWsClient;
pub use public::PublicWsClient;
