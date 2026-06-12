use crate::auth::AuthSigner;
use crate::rest::RestClient;
use gmo_coin_fx_core::{
    models::{
        AccountAsset, ActiveOrders, ApiStatus, CancelBulkOrderRequest, CancelOrderRequest,
        ChangeOrderRequest, CloseBulkOrderRequest, CloseOrderRequest, ExecutionsList, Kline, Order,
        OrderRequest, PositionSummaryList, PositionsList, SpeedOrderRequest, Symbol, Ticker,
        WsAuth,
    },
    Result,
};
use std::time::Duration;

/// GMO コイン FX API のメインクライアント。
///
/// [`GmoFxClient::builder()`] で構築してください。
/// 認証が不要なパブリック API と、API キーが必要なプライベート API の両方をサポートします。
#[derive(Clone)]
pub struct GmoFxClient {
    rest: RestClient,
}

/// リトライ制御のための設定。
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct RetryConfig {
    /// 最大リトライ回数
    pub max_retries: u32,
    /// 初回リトライ時の待機時間（ミリ秒）
    pub base_delay_ms: u64,
    /// 最大の待機時間（ミリ秒）
    pub max_delay_ms: u64,
}

impl RetryConfig {
    /// 新しい [`RetryConfig`] を生成します。
    pub fn new(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 2000,
        }
    }
}

/// [`GmoFxClient`] のビルダー。
pub struct GmoFxClientBuilder {
    api_key: Option<String>,
    secret_key: Option<String>,
    retry_config: Option<RetryConfig>,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    pool_max_idle_per_host: Option<usize>,
    base_url: Option<String>,
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

    /// 自動リトライを設定します。
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// HTTP リクエストのタイムアウトを設定します。
    ///
    /// # デフォルト値
    /// デフォルトは `None`（タイムアウトなし）です。
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// HTTP 接続のコネクションタイムアウトを設定します。
    ///
    /// # デフォルト値
    /// デフォルトは `None`（接続タイムアウトなし、OS/reqwest のデフォルトに依存）です。
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = Some(connect_timeout);
        self
    }

    /// HTTP 接続プール内の各ホストごとの最大アイドル接続数を設定します。
    ///
    /// # デフォルト値
    /// デフォルトは `None`（`reqwest` のデフォルト値（制限なし）が適用されます）。
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = Some(max);
        self
    }

    /// サンドボックス環境やテスト用のモックサーバー向けに、ベース URL を上書き設定します。
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// [`GmoFxClient`] を構築します。
    pub fn build(self) -> GmoFxClient {
        let auth = match (self.api_key, self.secret_key) {
            (Some(api_key), Some(secret_key)) => Some(AuthSigner::new(api_key, secret_key)),
            _ => None,
        };
        GmoFxClient {
            rest: RestClient::new(
                auth,
                self.retry_config,
                self.timeout,
                self.connect_timeout,
                self.pool_max_idle_per_host,
                self.base_url,
            ),
        }
    }
}

impl GmoFxClient {
    /// ビルダーを作成します。
    pub fn builder() -> GmoFxClientBuilder {
        GmoFxClientBuilder {
            api_key: None,
            secret_key: None,
            retry_config: None,
            timeout: None,
            connect_timeout: None,
            pool_max_idle_per_host: None,
            base_url: None,
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

    /// 注文を変更（価格等の変更）します。
    ///
    /// 認証必要。
    pub async fn change_order(&self, req: &ChangeOrderRequest) -> Result<()> {
        let _res: serde_json::Value = self.rest.private_post("/v1/changeOrder", req).await?;
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

    /// 有効注文一覧を自動でページネーション（prevId）しながら取得するストリームを生成します。
    ///
    /// # 引数
    /// - `symbol` — 銘柄でフィルタリング（省略可）
    pub fn active_orders_stream(&self, symbol: Option<&str>) -> ActiveOrdersStream {
        ActiveOrdersStream::new(self.clone(), symbol.map(|s| s.to_string()))
    }
}

/// 有効注文を自動でページネーションしながら取得するための非同期ストリームヘルパー。
pub struct ActiveOrdersStream {
    client: GmoFxClient,
    symbol: Option<String>,
    prev_id: Option<u64>,
    is_finished: bool,
}

impl ActiveOrdersStream {
    /// 新しい [`ActiveOrdersStream`] を生成します。
    pub fn new(client: GmoFxClient, symbol: Option<String>) -> Self {
        Self {
            client,
            symbol,
            prev_id: None,
            is_finished: false,
        }
    }

    /// 次のページの有効注文一覧を取得します。
    ///
    /// 取得結果が空の場合、または既に全件取得済みの場合は `None` を返します。
    pub async fn next(&mut self) -> Result<Option<ActiveOrders>> {
        if self.is_finished {
            return Ok(None);
        }

        let res = self
            .client
            .active_orders(self.symbol.as_deref(), self.prev_id, None)
            .await?;

        if res.list.is_empty() {
            self.is_finished = true;
            return Ok(None);
        }

        if let Some(last_order) = res.list.last() {
            self.prev_id = Some(last_order.order_id);
        } else {
            self.is_finished = true;
        }

        Ok(Some(res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default_base_urls() {
        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .build();
        assert_eq!(
            client.rest.public.base_url,
            "https://forex-api.coin.z.com/public"
        );
        assert_eq!(
            client.rest.private.as_ref().unwrap().base_url,
            "https://forex-api.coin.z.com/private"
        );
    }

    #[test]
    fn test_builder_custom_base_url() {
        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .base_url("http://localhost:8080")
            .build();
        assert_eq!(client.rest.public.base_url, "http://localhost:8080/public");
        assert_eq!(
            client.rest.private.as_ref().unwrap().base_url,
            "http://localhost:8080/private"
        );
    }

    async fn start_mock_server() -> (tokio::net::TcpListener, String) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}", port);
        (listener, url)
    }

    async fn handle_connection(mut stream: tokio::net::TcpStream, status_code: u16, body: &str) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut buf = [0; 1024];
        let _ = stream.read(&mut buf).await;
        let response = format!(
            "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status_code,
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;
    }

    #[tokio::test]
    async fn test_active_orders_stream_pagination() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            // First page request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{
                    "status": 0,
                    "data": {
                        "list": [
                            {
                                "rootOrderId": 1001,
                                "orderId": 1001,
                                "symbol": "USD_JPY",
                                "side": "BUY",
                                "orderType": "LIMIT",
                                "executionType": "LIMIT",
                                "settleType": "OPEN",
                                "size": "10000",
                                "price": "150.00",
                                "status": "ORDERED",
                                "timestamp": "2026-06-10T22:40:13Z"
                            },
                            {
                                "rootOrderId": 1002,
                                "orderId": 1002,
                                "symbol": "USD_JPY",
                                "side": "BUY",
                                "orderType": "LIMIT",
                                "executionType": "LIMIT",
                                "settleType": "OPEN",
                                "size": "10000",
                                "price": "150.00",
                                "status": "ORDERED",
                                "timestamp": "2026-06-10T22:40:13Z"
                            }
                        ]
                    }
                }"#;
                handle_connection(stream, 200, body).await;
            }

            // Second page request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{"status": 0, "data": {"list": []}}"#;
                handle_connection(stream, 200, body).await;
            }
        });

        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .base_url(&url)
            .build();

        let mut stream = client.active_orders_stream(Some("USD_JPY"));

        // First page
        let page1 = stream.next().await.unwrap().expect("should return page 1");
        assert_eq!(page1.list.len(), 2);
        assert_eq!(page1.list[0].order_id, 1001);
        assert_eq!(page1.list[1].order_id, 1002);

        // Second page
        let page2 = stream.next().await.unwrap();
        assert!(page2.is_none());

        // Subsequent page call should keep returning None
        let page3 = stream.next().await.unwrap();
        assert!(page3.is_none());
    }

    #[test]
    fn test_builder_pool_max_idle_per_host() {
        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .pool_max_idle_per_host(10)
            .build();
        assert!(client.rest.private.is_some());
    }
}
