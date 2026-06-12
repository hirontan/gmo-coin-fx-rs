use gmo_coin_fx_core::{models::ApiResponse, GmoFxError, Result};
use serde::de::DeserializeOwned;

pub(crate) const PUBLIC_BASE_URL: &str = "https://forex-api.coin.z.com/public";

/// 認証不要なパブリック REST エンドポイントのクライアント。
#[derive(Clone)]
pub struct PublicRestClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) retry_config: Option<crate::gateway::RetryConfig>,
}

impl PublicRestClient {
    /// 新しい [`PublicRestClient`] を生成します。
    pub fn new(
        retry_config: Option<crate::gateway::RetryConfig>,
        timeout: Option<std::time::Duration>,
        connect_timeout: Option<std::time::Duration>,
        pool_max_idle_per_host: Option<usize>,
        base_url: Option<String>,
    ) -> Self {
        let mut builder = reqwest::Client::builder();
        if let Some(t) = timeout {
            builder = builder.timeout(t);
        }
        if let Some(ct) = connect_timeout {
            builder = builder.connect_timeout(ct);
        }
        if let Some(max_idle) = pool_max_idle_per_host {
            builder = builder.pool_max_idle_per_host(max_idle);
        }
        let http = builder.build().expect("failed to build reqwest client");

        let resolved_base_url = match base_url {
            Some(url) => format!("{}/public", url.trim_end_matches('/')),
            None => PUBLIC_BASE_URL.to_string(),
        };

        Self {
            http,
            base_url: resolved_base_url,
            retry_config,
        }
    }

    /// GET リクエストを送信し、レスポンスを `T` にデシリアライズします。
    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let mut attempt = 0;
        let max_retries = self.retry_config.map(|c| c.max_retries).unwrap_or(0);

        loop {
            let res_result = self.http.get(&url).send().await;

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

            return Ok(api_res.data);
        }
    }
}

impl Default for PublicRestClient {
    fn default() -> Self {
        Self::new(None, None, None, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::RetryConfig;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn start_mock_server() -> (tokio::net::TcpListener, String) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}", port);
        (listener, url)
    }

    async fn handle_connection(mut stream: tokio::net::TcpStream, status_code: u16, body: &str) {
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
    async fn test_retry_on_5xx_success() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                handle_connection(stream, 500, "{}").await;
            }
            if let Ok((stream, _)) = listener.accept().await {
                handle_connection(stream, 503, "{}").await;
            }
            if let Ok((stream, _)) = listener.accept().await {
                handle_connection(
                    stream,
                    200,
                    r#"{"status": 0, "messages": [], "data": "success"}"#,
                )
                .await;
            }
        });

        let mut client = PublicRestClient::new(
            Some(RetryConfig {
                max_retries: 3,
                base_delay_ms: 1,
                max_delay_ms: 10,
            }),
            None,
            None,
            None,
            None,
        );
        client.base_url = url;

        let res: Result<String> = client.get("/test").await;
        assert_eq!(res.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_exceeded_error() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                handle_connection(stream, 500, "{}").await;
            }
            if let Ok((stream, _)) = listener.accept().await {
                handle_connection(stream, 500, "{}").await;
            }
            if let Ok((stream, _)) = listener.accept().await {
                handle_connection(stream, 500, "{}").await;
            }
        });

        let mut client = PublicRestClient::new(
            Some(RetryConfig {
                max_retries: 2,
                base_delay_ms: 1,
                max_delay_ms: 10,
            }),
            None,
            None,
            None,
            None,
        );
        client.base_url = url;

        let res: Result<String> = client.get("/test").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_retry_on_connection_error_exceeded() {
        let mut client = PublicRestClient::new(
            Some(RetryConfig {
                max_retries: 2,
                base_delay_ms: 10,
                max_delay_ms: 30,
            }),
            None,
            None,
            None,
            None,
        );
        // Use a port that is not listening
        client.base_url = "http://127.0.0.1:59999".to_string();

        let start = std::time::Instant::now();
        let res: Result<String> = client.get("/test").await;
        let duration = start.elapsed();

        assert!(res.is_err());
        // Since max_retries = 2, it should sleep for 10ms + 20ms = 30ms.
        // We assert that the duration is at least 25ms.
        assert!(
            duration.as_millis() >= 25,
            "Duration was {}ms",
            duration.as_millis()
        );
    }

    #[tokio::test]
    async fn test_client_timeout() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                handle_connection(
                    stream,
                    200,
                    r#"{"status": 0, "messages": [], "data": "success"}"#,
                )
                .await;
            }
        });

        let mut client = PublicRestClient::new(
            None,
            Some(tokio::time::Duration::from_millis(30)),
            None,
            None,
            None,
        );
        client.base_url = url;

        let start = std::time::Instant::now();
        let res: Result<String> = client.get("/test").await;
        let duration = start.elapsed();

        assert!(res.is_err());
        assert!(
            duration.as_millis() < 90,
            "Duration was {}ms",
            duration.as_millis()
        );
    }

    #[tokio::test]
    async fn test_custom_base_url() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                handle_connection(
                    stream,
                    200,
                    r#"{"status": 0, "messages": [], "data": "custom_base"}"#,
                )
                .await;
            }
        });

        let client = PublicRestClient::new(None, None, None, None, Some(url.clone()));
        assert_eq!(client.base_url, format!("{}/public", url));

        let res: Result<String> = client.get("/test").await;
        assert_eq!(res.unwrap(), "custom_base");
    }
}
