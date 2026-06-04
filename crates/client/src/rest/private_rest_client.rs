use gmo_coin_fx_core::{models::ApiResponse, GmoFxError, Result};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::AuthSigner;

pub(crate) const PRIVATE_BASE_URL: &str = "https://forex-api.coin.z.com/private";

/// 認証が必要なプライベート REST エンドポイントのクライアント。
#[derive(Clone)]
pub struct PrivateRestClient {
    pub(crate) http: reqwest::Client,
    pub(crate) auth: AuthSigner,
    pub(crate) base_url: String,
    pub(crate) retry_config: Option<crate::gateway::RetryConfig>,
}

impl PrivateRestClient {
    /// 新しい [`PrivateRestClient`] を生成します。
    pub fn new(auth: AuthSigner, retry_config: Option<crate::gateway::RetryConfig>) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth,
            base_url: PRIVATE_BASE_URL.to_string(),
            retry_config,
        }
    }

    async fn request_with_retry<T, B>(
        &self,
        method: Method,
        path: &str,
        query: Option<&[(&str, String)]>,
        body: Option<&B>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let mut attempt = 0;
        let max_retries = self.retry_config.map(|c| c.max_retries).unwrap_or(0);
        let url = format!("{}{}", self.base_url, path);

        loop {
            let timestamp = current_timestamp_millis();
            let body_text = match body {
                Some(b) => serde_json::to_string(b).map_err(|e| GmoFxError::Json(e.to_string()))?,
                None => "".to_string(),
            };
            let headers = self
                .auth
                .sign(&timestamp, method.as_str(), path, &body_text);

            let mut req = self
                .http
                .request(method.clone(), &url)
                .header("API-KEY", headers.api_key)
                .header("API-TIMESTAMP", headers.api_timestamp)
                .header("API-SIGN", headers.api_sign);

            if let Some(q) = query {
                req = req.query(q);
            }

            if body.is_some() {
                req = req
                    .header("content-type", "application/json")
                    .body(body_text);
            }

            let res_result = req.send().await;

            let is_retryable = match &res_result {
                Err(e) => e.is_timeout() || e.is_connect(),
                Ok(res) => res.status().is_server_error(),
            };

            if is_retryable && attempt < max_retries {
                let config = self.retry_config.as_ref().unwrap();
                let delay_ms = (config.base_delay_ms * 2u64.pow(attempt)).min(config.max_delay_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                attempt += 1;
                continue;
            }

            let res = res_result.map_err(|e| GmoFxError::Http(e.to_string()))?;
            return parse_response(res).await;
        }
    }

    /// 署名付き GET リクエストを送信します。
    pub async fn get<T>(&self, path: &str, query: Option<&[(&str, String)]>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request_with_retry(Method::GET, path, query, None::<&()>)
            .await
    }

    /// 署名付き POST リクエストを送信します。
    pub async fn post<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request_with_retry(Method::POST, path, None, Some(body))
            .await
    }

    /// 署名付き PUT リクエストを送信します。
    pub async fn put<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request_with_retry(Method::PUT, path, None, Some(body))
            .await
    }

    /// 署名付き DELETE リクエストを送信します。
    pub async fn delete<T>(&self, path: &str, query: Option<&[(&str, String)]>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request_with_retry(Method::DELETE, path, query, None::<&()>)
            .await
    }
}

async fn parse_response<T>(res: reqwest::Response) -> Result<T>
where
    T: DeserializeOwned,
{
    let api_res: ApiResponse<T> = res
        .json()
        .await
        .map_err(|e| GmoFxError::Json(e.to_string()))?;

    if api_res.status != 0 {
        return Err(GmoFxError::Api {
            status: api_res.status,
            messages: api_res.messages,
        });
    }

    Ok(api_res.data)
}

fn current_timestamp_millis() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH");
    now.as_millis().to_string()
}
