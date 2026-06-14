use futures_util::{SinkExt, StreamExt};
use gmo_coin_fx_client::GmoFxClient;
use gmo_coin_fx_core::{
    models::ws::SubscribeCommand, models::ws_events::PrivateWsMessage, GmoFxError, Result,
};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub const PRIVATE_WS_URL: &str = "wss://forex-api.coin.z.com/ws/private/v1";

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub struct PrivateWsClient {
    ws_stream: WsStream,
    renew_task: Option<tokio::task::JoinHandle<()>>,
    client: GmoFxClient,
    subscriptions: HashSet<String>,
    ping_interval: tokio::time::Interval,
    ping_interval_duration: Duration,
    ping_pending: bool,
    ws_url_base: String,
}

impl PrivateWsClient {
    pub async fn connect(client: GmoFxClient) -> Result<Self> {
        Self::connect_with_ping_interval(client, Duration::from_secs(30)).await
    }

    pub async fn connect_with_ping_interval(
        client: GmoFxClient,
        ping_interval: Duration,
    ) -> Result<Self> {
        Self::connect_with_url(client, PRIVATE_WS_URL, ping_interval).await
    }

    pub async fn connect_with_url(
        client: GmoFxClient,
        url_base: &str,
        ping_interval: Duration,
    ) -> Result<Self> {
        let (ws_stream, renew_task) = Self::connect_stream_to_url(&client, url_base).await?;
        let mut interval = tokio::time::interval(ping_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;

        Ok(Self {
            ws_stream,
            renew_task: Some(renew_task),
            client,
            subscriptions: HashSet::new(),
            ping_interval: interval,
            ping_interval_duration: ping_interval,
            ping_pending: false,
            ws_url_base: url_base.to_string(),
        })
    }

    async fn connect_stream_to_url(
        client: &GmoFxClient,
        url_base: &str,
    ) -> Result<(WsStream, tokio::task::JoinHandle<()>)> {
        let auth = client.ws_auth_post().await?;
        let url_str = format!("{}/{}", url_base, auth.token);
        let url = Url::parse(&url_str).map_err(|e| GmoFxError::Http(e.to_string()))?;

        let (ws_stream, _) = connect_async(url.as_str())
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;

        // Start auto-renew task (renew every 30 minutes)
        let renew_client = client.clone();
        let renew_task = tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30 * 60)).await;
                if let Err(e) = renew_client.ws_auth_put().await {
                    eprintln!("Failed to renew ws-auth token: {:?}", e);
                }
            }
        });

        Ok((ws_stream, renew_task))
    }

    pub async fn subscribe(&mut self, channel: &str) -> Result<()> {
        let cmd = SubscribeCommand::new(channel);
        let msg = serde_json::to_string(&cmd).map_err(|e| GmoFxError::Json(e.to_string()))?;

        self.ws_stream
            .send(Message::Text(msg.into()))
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;

        self.subscriptions.insert(channel.to_string());
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<PrivateWsMessage>> {
        loop {
            let msg_fut = self.ws_stream.next();
            let tick_fut = self.ping_interval.tick();

            tokio::select! {
                msg = msg_fut => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            let event: PrivateWsMessage =
                                serde_json::from_str(&text).map_err(|e| GmoFxError::Json(e.to_string()))?;
                            return Ok(Some(event));
                        }
                        Some(Ok(Message::Pong(_))) => {
                            self.ping_pending = false;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            let _ = self.ws_stream.send(Message::Pong(data)).await;
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            self.reconnect().await?;
                        }
                        Some(Err(e)) => {
                            eprintln!("WebSocket error: {:?}, attempting reconnect...", e);
                            self.reconnect().await?;
                        }
                        _ => {}
                    }
                }
                _ = tick_fut => {
                    if self.ping_pending {
                        eprintln!("Ping timeout: no pong received. Reconnecting...");
                        self.reconnect().await?;
                    } else {
                        if let Err(e) = self.ws_stream.send(Message::Ping(vec![].into())).await {
                            eprintln!("Failed to send ping: {:?}, attempting reconnect...", e);
                            self.reconnect().await?;
                        } else {
                            self.ping_pending = true;
                        }
                    }
                }
            }
        }
    }

    async fn reconnect(&mut self) -> Result<()> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let backoff = if self.ws_url_base.contains("127.0.0.1")
                || self.ws_url_base.contains("localhost")
            {
                Duration::from_millis(10)
            } else {
                Duration::from_secs(std::cmp::min(2u64.pow(attempts), 60))
            };
            println!("Attempting to reconnect in {:?}...", backoff);
            sleep(backoff).await;

            if let Some(task) = self.renew_task.take() {
                task.abort();
            }

            match Self::connect_stream_to_url(&self.client, &self.ws_url_base).await {
                Ok((stream, task)) => {
                    self.ws_stream = stream;
                    self.renew_task = Some(task);
                    self.ping_pending = false;
                    self.ping_interval = tokio::time::interval(self.ping_interval_duration);
                    self.ping_interval.tick().await;
                    println!("Reconnected successfully.");

                    let subs = self.subscriptions.clone();
                    for channel in subs {
                        let cmd = SubscribeCommand::new(&channel);
                        if let Ok(msg) = serde_json::to_string(&cmd) {
                            let _ = self.ws_stream.send(Message::Text(msg.into())).await;
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Reconnect failed: {:?}", e);
                }
            }
        }
    }
}

impl Drop for PrivateWsClient {
    fn drop(&mut self) {
        if let Some(task) = self.renew_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::Mutex;
    use tokio_tungstenite::accept_async;

    #[tokio::test]
    async fn test_private_ws_liveness() {
        // 1. Mock HTTP server
        let http_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let http_port = http_listener.local_addr().unwrap().port();
        let http_url = format!("http://127.0.0.1:{}", http_port);

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = http_listener.accept().await {
                let mut buf = [0; 1024];
                let _ = stream.read(&mut buf).await;
                let body = r#"{"status":0,"messages":[],"data":{"token":"test-token"}}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.flush().await;
            }
        });

        // 2. Mock WS server
        let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_port = ws_listener.local_addr().unwrap().port();
        let ws_url = format!("ws://127.0.0.1:{}", ws_port);

        let ping_received = Arc::new(Mutex::new(false));
        let ping_received_clone = ping_received.clone();

        tokio::spawn(async move {
            if let Ok((stream, _)) = ws_listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                while let Some(msg) = ws_stream.next().await {
                    match msg.unwrap() {
                        Message::Ping(data) => {
                            *ping_received_clone.lock().await = true;
                            ws_stream.send(Message::Pong(data)).await.unwrap();

                            // Send an orderEvents message after ping-pong is handled
                            let event_json = r#"{"channel":"orderEvents","rootOrderId":123,"orderId":456,"symbol":"USD_JPY","settleType":"OPEN","orderType":"NORMAL","executionType":"LIMIT","side":"BUY","orderStatus":"ORDERED","orderTimestamp":"2026-06-14T22:00:00Z","orderPrice":"150.0","orderSize":"10000","msgType":"ER"}"#;
                            ws_stream
                                .send(Message::Text(event_json.into()))
                                .await
                                .unwrap();
                        }
                        Message::Text(_) => {}
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
        });

        // 3. Build GmoFxClient
        let client = GmoFxClient::builder()
            .credentials("test_api_key", "test_secret_key")
            .base_url(http_url)
            .build();

        // 4. Connect PrivateWsClient
        let mut ws_client =
            PrivateWsClient::connect_with_url(client, &ws_url, Duration::from_millis(50))
                .await
                .unwrap();

        ws_client.subscribe("orderEvents").await.unwrap();

        let msg = ws_client.next_message().await.unwrap();
        assert!(msg.is_some());
        if let Some(PrivateWsMessage::Order(o)) = msg {
            assert_eq!(o.symbol, "USD_JPY");
        } else {
            panic!("Expected Order event");
        }

        assert!(*ping_received.lock().await);
        assert!(!ws_client.ping_pending);
    }

    #[tokio::test]
    async fn test_private_ws_timeout_reconnect() {
        // 1. Mock HTTP server that handles 2 tokens
        let http_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let http_port = http_listener.local_addr().unwrap().port();
        let http_url = format!("http://127.0.0.1:{}", http_port);

        tokio::spawn(async move {
            for i in 1..=2 {
                if let Ok((mut stream, _)) = http_listener.accept().await {
                    let mut buf = [0; 1024];
                    let _ = stream.read(&mut buf).await;
                    let body = format!(
                        r#"{{"status":0,"messages":[],"data":{{"token":"token-{}"}}}}"#,
                        i
                    );
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.flush().await;
                }
            }
        });

        // 2. Mock WS server
        let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_port = ws_listener.local_addr().unwrap().port();
        let ws_url = format!("ws://127.0.0.1:{}", ws_port);

        let connections = Arc::new(Mutex::new(0));
        let connections_clone = connections.clone();

        tokio::spawn(async move {
            // First WS connection
            if let Ok((stream, _)) = ws_listener.accept().await {
                *connections_clone.lock().await += 1;
                let mut ws_stream = accept_async(stream).await.unwrap();
                while let Some(msg) = ws_stream.next().await {
                    if let Message::Ping(_) = msg.unwrap() {
                        // Ignore Ping to force timeout
                    }
                }
            }

            // Second WS connection
            if let Ok((stream, _)) = ws_listener.accept().await {
                *connections_clone.lock().await += 1;
                let mut ws_stream = accept_async(stream).await.unwrap();
                while let Some(msg) = ws_stream.next().await {
                    if let Message::Text(_) = msg.unwrap() {
                        // On subscription, send the orderEvents message
                        let event_json = r#"{"channel":"orderEvents","rootOrderId":123,"orderId":456,"symbol":"USD_JPY","settleType":"OPEN","orderType":"NORMAL","executionType":"LIMIT","side":"BUY","orderStatus":"ORDERED","orderTimestamp":"2026-06-14T22:00:00Z","orderPrice":"150.0","orderSize":"10000","msgType":"ER"}"#;
                        ws_stream
                            .send(Message::Text(event_json.into()))
                            .await
                            .unwrap();
                    }
                }
            }
        });

        // 3. Build GmoFxClient
        let client = GmoFxClient::builder()
            .credentials("test_api_key", "test_secret_key")
            .base_url(http_url)
            .build();

        // 4. Connect PrivateWsClient
        let mut ws_client =
            PrivateWsClient::connect_with_url(client, &ws_url, Duration::from_millis(50))
                .await
                .unwrap();

        ws_client.subscribe("orderEvents").await.unwrap();

        let msg = ws_client.next_message().await.unwrap();
        assert!(msg.is_some());
        if let Some(PrivateWsMessage::Order(o)) = msg {
            assert_eq!(o.symbol, "USD_JPY");
        } else {
            panic!("Expected Order event");
        }

        assert_eq!(*connections.lock().await, 2);
    }
}
