use gmo_coin_fx_core::{models::ApiResponse, GmoFxError, Result};
use serde::de::DeserializeOwned;

pub(crate) const PUBLIC_BASE_URL: &str = "https://forex-api.coin.z.com/public";

/// 認証不要なパブリック REST エンドポイントのクライアント。
#[derive(Clone)]
pub struct PublicRestClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
}

impl PublicRestClient {
    /// 新しい [`PublicRestClient`] を生成します。
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: PUBLIC_BASE_URL.to_string(),
        }
    }

    /// GET リクエストを送信し、レスポンスを `T` にデシリアライズします。
    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let res = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
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
}

impl Default for PublicRestClient {
    fn default() -> Self {
        Self::new()
    }
}
