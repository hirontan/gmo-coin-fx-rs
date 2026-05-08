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
}

impl PrivateRestClient {
    /// 新しい [`PrivateRestClient`] を生成します。
    pub fn new(auth: AuthSigner) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth,
            base_url: PRIVATE_BASE_URL.to_string(),
        }
    }

    /// 署名付き GET リクエストを送信します。
    pub async fn get<T>(&self, path: &str, query: Option<&[(&str, String)]>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let timestamp = current_timestamp_millis();
        let headers = self.auth.sign(&timestamp, "GET", path, "");
        let url = format!("{}{}", self.base_url, path);

        let mut req = self
            .http
            .request(Method::GET, url)
            .header("API-KEY", headers.api_key)
            .header("API-TIMESTAMP", headers.api_timestamp)
            .header("API-SIGN", headers.api_sign);

        if let Some(query) = query {
            req = req.query(query);
        }

        let res = req
            .send()
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
        parse_response(res).await
    }

    /// 署名付き POST リクエストを送信します。
    pub async fn post<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let timestamp = current_timestamp_millis();
        let body_text =
            serde_json::to_string(body).map_err(|e| GmoFxError::Json(e.to_string()))?;
        let headers = self.auth.sign(&timestamp, "POST", path, &body_text);
        let url = format!("{}{}", self.base_url, path);

        let res = self
            .http
            .post(url)
            .header("content-type", "application/json")
            .header("API-KEY", headers.api_key)
            .header("API-TIMESTAMP", headers.api_timestamp)
            .header("API-SIGN", headers.api_sign)
            .body(body_text)
            .send()
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
        parse_response(res).await
    }

    /// 署名付き PUT リクエストを送信します。
    pub async fn put<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let timestamp = current_timestamp_millis();
        let body_text =
            serde_json::to_string(body).map_err(|e| GmoFxError::Json(e.to_string()))?;
        let headers = self.auth.sign(&timestamp, "PUT", path, &body_text);
        let url = format!("{}{}", self.base_url, path);

        let res = self
            .http
            .put(url)
            .header("content-type", "application/json")
            .header("API-KEY", headers.api_key)
            .header("API-TIMESTAMP", headers.api_timestamp)
            .header("API-SIGN", headers.api_sign)
            .body(body_text)
            .send()
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
        parse_response(res).await
    }

    /// 署名付き DELETE リクエストを送信します。
    pub async fn delete<T>(&self, path: &str, query: Option<&[(&str, String)]>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let timestamp = current_timestamp_millis();
        let headers = self.auth.sign(&timestamp, "DELETE", path, "");
        let url = format!("{}{}", self.base_url, path);

        let mut req = self
            .http
            .request(Method::DELETE, url)
            .header("API-KEY", headers.api_key)
            .header("API-TIMESTAMP", headers.api_timestamp)
            .header("API-SIGN", headers.api_sign);

        if let Some(query) = query {
            req = req.query(query);
        }

        let res = req
            .send()
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
        parse_response(res).await
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
