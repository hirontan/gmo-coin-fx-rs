//! # gmo-coin-fx-core
//!
//! GMO コイン FX API のドメインモデル・エラー型・認証署名ロジックを提供するクレートです。
//! このクレートは [`gmo-coin-fx-client`](https://github.com/hirontan/gmo-coin-fx-rs) および
//! [`gmo-coin-fx-ws`](https://github.com/hirontan/gmo-coin-fx-rs) の基盤となります。

pub mod error;
pub mod models;

pub use error::{ApiMessage, GmoFxError, Result};
