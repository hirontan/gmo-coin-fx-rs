use gmo_coin_fx_core::{models::ApiResponse, GmoFxError, Result};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::AuthSigner;

const PUBLIC_BASE_URL: &str = "https://forex-api.coin.z.com/public";
const PRIVATE_BASE_URL: &str = "https://forex-api.coin.z.com/private";

#[derive(Clone)]
pub struct RestClient {
    http: reqwest::Client,
    auth: Option<AuthSigner>,
    public_base_url: String,
    private_base_url: String,
}

impl RestClient {
    pub fn new(auth: Option<AuthSigner>) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth,
            public_base_url: PUBLIC_BASE_URL.to_string(),
            private_base_url: PRIVATE_BASE_URL.to_string(),
        }
    }

    pub async fn public_get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.public_base_url, path);
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

    pub async fn private_get<T>(&self, path: &str, query: Option<&[(&str, String)]>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let auth = self.auth.as_ref().ok_or(GmoFxError::MissingCredentials)?;
        let timestamp = current_timestamp_millis();
        let headers = auth.sign(&timestamp, "GET", path, "");

        let url = format!("{}{}", self.private_base_url, path);

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

    pub async fn private_post<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let auth = self.auth.as_ref().ok_or(GmoFxError::MissingCredentials)?;
        let timestamp = current_timestamp_millis();
        let body_text = serde_json::to_string(body).map_err(|e| GmoFxError::Json(e.to_string()))?;
        let headers = auth.sign(&timestamp, "POST", path, &body_text);

        let url = format!("{}{}", self.private_base_url, path);

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

    pub async fn private_put<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let auth = self.auth.as_ref().ok_or(GmoFxError::MissingCredentials)?;
        let timestamp = current_timestamp_millis();
        let body_text = serde_json::to_string(body).map_err(|e| GmoFxError::Json(e.to_string()))?;
        let headers = auth.sign(&timestamp, "PUT", path, &body_text);

        let url = format!("{}{}", self.private_base_url, path);

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

    pub async fn private_delete<T>(&self, path: &str, query: Option<&[(&str, String)]>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let auth = self.auth.as_ref().ok_or(GmoFxError::MissingCredentials)?;
        let timestamp = current_timestamp_millis();
        let headers = auth.sign(&timestamp, "DELETE", path, "");

        let url = format!("{}{}", self.private_base_url, path);

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

fn current_timestamp_millis() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH");

    now.as_millis().to_string()
}
