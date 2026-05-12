use crate::auth::AuthSigner;
use crate::rest::RestClient;
use gmo_coin_fx_core::{
    models::{
        AccountAsset, ActiveOrders, ApiStatus, CancelBulkOrderRequest, CancelOrderRequest,
        CloseBulkOrderRequest, CloseOrderRequest, ExecutionsList, Kline, Order, OrderRequest,
        PositionSummaryList, PositionsList, SpeedOrderRequest, Symbol, Ticker, WsAuth,
    },
    Result,
};

/// GMO コイン FX API のメインクライアント。
///
/// [`GmoFxClient::builder()`] で構築してください。
/// 認証が不要なパブリック API と、API キーが必要なプライベート API の両方をサポートします。
#[derive(Clone)]
pub struct GmoFxClient {
    rest: RestClient,
}

/// [`GmoFxClient`] のビルダー。
pub struct GmoFxClientBuilder {
    api_key: Option<String>,
    secret_key: Option<String>,
}

impl GmoFxClientBuilder {
    /// API キーとシークレットキーを設定します（プライベート API を使う場合に必須）。
    pub fn credentials(
        mut self,
        api_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.api_key = Some(api_key.into());
        self.secret_key = Some(secret_key.into());
        self
    }

    /// [`GmoFxClient`] を構築します。
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
    /// ビルダーを作成します。
    pub fn builder() -> GmoFxClientBuilder {
        GmoFxClientBuilder {
            api_key: None,
            secret_key: None,
        }
    }

    // ─── Public REST API ───────────────────────────────────────────────

    /// 全銘柄のティッカー情報を取得します。
    ///
    /// 認証不要。
    pub async fn ticker(&self) -> Result<Vec<Ticker>> {
        self.rest.public_get("/v1/ticker").await
    }

    /// サービスのステータスを取得します。
    ///
    /// 認証不要。
    pub async fn status(&self) -> Result<ApiStatus> {
        self.rest.public_get("/v1/status").await
    }

    /// ローソク足データを取得します。
    ///
    /// # 引数
    /// - `symbol` — 銘柄コード（例: `"USD_JPY"`）
    /// - `price_type` — 価格種別（`"BID"` または `"ASK"`）
    /// - `interval` — 時間足（例: `"1min"`, `"1hour"`, `"1day"`）
    /// - `date` — 取得日付（`"YYYYMMDD"` 形式）
    ///
    /// 認証不要。
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

    /// 取引可能な銘柄一覧を取得します。
    ///
    /// 認証不要。
    pub async fn symbols(&self) -> Result<Vec<Symbol>> {
        self.rest.public_get("/v1/symbols").await
    }

    // ─── Private REST API ──────────────────────────────────────────────

    /// 口座の資産残高を取得します。
    ///
    /// 認証必要。
    pub async fn assets(&self) -> Result<Vec<AccountAsset>> {
        self.rest.private_get("/v1/account/assets", None).await
    }

    /// 新規注文を発注します。
    ///
    /// 認証必要。
    pub async fn order(&self, req: &OrderRequest) -> Result<Vec<Order>> {
        self.rest.private_post("/v1/order", req).await
    }

    /// 有効注文の一覧を取得します。
    ///
    /// # 引数
    /// - `symbol` — 銘柄でフィルタリング（省略可）
    /// - `prev_id` — ページネーション用の前回最終 ID（省略可）
    /// - `count` — 取得件数（省略可、最大 100）
    ///
    /// 認証必要。
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

    /// 約定履歴を取得します。
    ///
    /// # 引数
    /// - `order_id` — 注文 ID で絞り込み（省略可）
    /// - `execution_id` — 約定 ID で絞り込み（省略可）
    ///
    /// 認証必要。
    pub async fn executions(
        &self,
        order_id: Option<u64>,
        execution_id: Option<u64>,
    ) -> Result<ExecutionsList> {
        let mut query = Vec::new();
        if let Some(id) = order_id {
            query.push(("orderId", id.to_string()));
        }
        if let Some(id) = execution_id {
            query.push(("executionId", id.to_string()));
        }
        self.rest
            .private_get(
                "/v1/executions",
                if query.is_empty() { None } else { Some(&query) },
            )
            .await
    }

    /// 最新の約定履歴を取得します。
    ///
    /// # 引数
    /// - `symbol` — 銘柄でフィルタリング（省略可）
    /// - `count` — 取得件数（省略可）
    ///
    /// 認証必要。
    pub async fn latest_executions(
        &self,
        symbol: Option<&str>,
        count: Option<u32>,
    ) -> Result<ExecutionsList> {
        let mut query = Vec::new();
        if let Some(sym) = symbol {
            query.push(("symbol", sym.to_string()));
        }
        if let Some(c) = count {
            query.push(("count", c.to_string()));
        }
        self.rest
            .private_get(
                "/v1/latestExecutions",
                if query.is_empty() { None } else { Some(&query) },
            )
            .await
    }

    /// 保有中の建玉一覧を取得します。
    ///
    /// 認証必要。
    pub async fn open_positions(
        &self,
        symbol: Option<&str>,
        count: Option<u32>,
    ) -> Result<PositionsList> {
        let mut query = Vec::new();
        if let Some(sym) = symbol {
            query.push(("symbol", sym.to_string()));
        }
        if let Some(c) = count {
            query.push(("count", c.to_string()));
        }
        self.rest
            .private_get(
                "/v1/openPositions",
                if query.is_empty() { None } else { Some(&query) },
            )
            .await
    }

    /// 建玉サマリーを取得します。
    ///
    /// 認証必要。
    pub async fn position_summary(&self, symbol: Option<&str>) -> Result<PositionSummaryList> {
        let mut query = Vec::new();
        if let Some(sym) = symbol {
            query.push(("symbol", sym.to_string()));
        }
        self.rest
            .private_get(
                "/v1/positionSummary",
                if query.is_empty() { None } else { Some(&query) },
            )
            .await
    }

    /// 注文をキャンセルします。
    ///
    /// 認証必要。
    pub async fn cancel_order(&self, req: &CancelOrderRequest) -> Result<()> {
        let _res: serde_json::Value = self.rest.private_post("/v1/cancelOrder", req).await?;
        Ok(())
    }

    /// 複数の注文を一括キャンセルします。
    ///
    /// 認証必要。戻り値はキャンセルに成功した注文 ID のリストです。
    pub async fn cancel_bulk_order(&self, req: &CancelBulkOrderRequest) -> Result<Vec<u64>> {
        self.rest.private_post("/v1/cancelBulkOrder", req).await
    }

    /// 建玉の決済注文を発注します。
    ///
    /// 認証必要。
    pub async fn close_order(&self, req: &CloseOrderRequest) -> Result<Vec<Order>> {
        self.rest.private_post("/v1/closeOrder", req).await
    }

    /// 複数建玉の決済注文を一括発注します。
    ///
    /// 認証必要。
    pub async fn close_bulk_order(&self, req: &CloseBulkOrderRequest) -> Result<Vec<Order>> {
        self.rest.private_post("/v1/closeBulkOrder", req).await
    }

    /// スピード注文（成行の即時決済）を発注します。
    ///
    /// 認証必要。
    pub async fn speed_order(&self, req: &SpeedOrderRequest) -> Result<Vec<Order>> {
        self.rest.private_post("/v1/speedOrder", req).await
    }

    // ─── WebSocket Auth ────────────────────────────────────────────────

    /// WebSocket 認証トークンを新規発行します。
    ///
    /// 認証必要。
    pub async fn ws_auth_post(&self) -> Result<WsAuth> {
        let empty = serde_json::json!({});
        self.rest.private_post("/v1/ws-auth", &empty).await
    }

    /// WebSocket 認証トークンの有効期限を延長します（30 分）。
    ///
    /// 認証必要。
    pub async fn ws_auth_put(&self) -> Result<()> {
        let empty = serde_json::json!({});
        let _res: serde_json::Value = self.rest.private_put("/v1/ws-auth", &empty).await?;
        Ok(())
    }

    /// WebSocket 認証トークンを破棄します。
    ///
    /// 認証必要。
    pub async fn ws_auth_delete(&self) -> Result<()> {
        let _res: serde_json::Value = self.rest.private_delete("/v1/ws-auth", None).await?;
        Ok(())
    }
}
