//! # gmo-coin-fx-client
//!
//! GMO コイン FX の REST API を非同期で呼び出すクライアントクレートです。
//!
//! ## 使い方
//!
//! ```rust,no_run
//! use gmo_coin_fx_client::GmoFxClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // パブリック API（認証不要）
//!     let client = GmoFxClient::builder().build();
//!     let tickers = client.ticker().await?;
//!     println!("{:?}", tickers);
//!
//!     // プライベート API（API キーが必要）
//!     let auth_client = GmoFxClient::builder()
//!         .credentials("YOUR_API_KEY", "YOUR_SECRET")
//!         .build();
//!     let assets = auth_client.assets().await?;
//!     println!("{:?}", assets);
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod gateway;
pub mod rest;

pub use gateway::{GmoFxClient, GmoFxClientBuilder};
