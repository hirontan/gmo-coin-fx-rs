use crate::auth::AuthSigner;
use crate::rest::RestClient;
use gmo_coin_fx_core::{
    models::{AccountAsset, ActiveOrders, Order, OrderRequest, Ticker, ApiStatus, Kline, Symbol},
    Result,
};

#[derive(Clone)]
pub struct GmoFxClient {
    rest: RestClient,
}

pub struct GmoFxClientBuilder {
    api_key: Option<String>,
    secret_key: Option<String>,
}

impl GmoFxClientBuilder {
    pub fn credentials(
        mut self,
        api_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.api_key = Some(api_key.into());
        self.secret_key = Some(secret_key.into());
        self
    }

    pub fn build(self) -> GmoFxClient {
        let auth = match (self.api_key, self.secret_key) {
            (Some(api_key), Some(secret_key)) => Some(AuthSigner::new(api_key, secret_key)),
            _ => None,
        };

        GmoFxClient {
            rest: RestClient::new(auth),
        }
    }
}

impl GmoFxClient {
    pub fn builder() -> GmoFxClientBuilder {
        GmoFxClientBuilder {
            api_key: None,
            secret_key: None,
        }
    }

    pub async fn ticker(&self) -> Result<Vec<Ticker>> {
        self.rest.public_get("/v1/ticker").await
    }

    pub async fn status(&self) -> Result<ApiStatus> {
        self.rest.public_get("/v1/status").await
    }

    pub async fn klines(
        &self,
        symbol: &str,
        price_type: &str,
        interval: &str,
        date: &str,
    ) -> Result<Vec<Kline>> {
        let path = format!(
            "/v1/klines?symbol={}&priceType={}&interval={}&date={}",
            symbol, price_type, interval, date
        );
        self.rest.public_get(&path).await
    }

    pub async fn symbols(&self) -> Result<Vec<Symbol>> {
        self.rest.public_get("/v1/symbols").await
    }

    pub async fn assets(&self) -> Result<Vec<AccountAsset>> {
        self.rest.private_get("/v1/account/assets", None).await
    }

    pub async fn order(&self, req: &OrderRequest) -> Result<Vec<Order>> {
        self.rest.private_post("/v1/order", req).await
    }

    pub async fn active_orders(
        &self,
        symbol: Option<&str>,
        prev_id: Option<u64>,
        count: Option<u32>,
    ) -> Result<ActiveOrders> {
        let mut query = Vec::new();

        if let Some(symbol) = symbol {
            query.push(("symbol", symbol.to_string()));
        }
        if let Some(prev_id) = prev_id {
            query.push(("prevId", prev_id.to_string()));
        }
        if let Some(count) = count {
            query.push(("count", count.to_string()));
        }

        self.rest
            .private_get(
                "/v1/activeOrders",
                if query.is_empty() { None } else { Some(&query) },
            )
            .await
    }
}
