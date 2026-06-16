use futures_util::{SinkExt, StreamExt};
use gmo_coin_fx_core::{
    models::ws::SubscribeCommand, models::ws_events::PublicWsMessage, GmoFxError, Result,
};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub const PUBLIC_WS_URL: &str = "wss://forex-api.coin.z.com/ws/public/v1";

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub struct PublicWsClient {
    ws_stream: WsStream,
    subscriptions: HashSet<(String, Option<String>)>,
    ping_interval: tokio::time::Interval,
    ping_interval_duration: Duration,
    ping_pending: bool,
    ws_url: String,
    on_connect: Option<Box<dyn Fn() + Send + Sync + 'static>>,
    on_disconnect: Option<Box<dyn Fn() + Send + Sync + 'static>>,
}

pub struct PublicWsClientBuilder {
    url: String,
    ping_interval: Duration,
    on_connect: Option<Box<dyn Fn() + Send + Sync + 'static>>,
    on_disconnect: Option<Box<dyn Fn() + Send + Sync + 'static>>,
}

impl Default for PublicWsClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PublicWsClientBuilder {
    pub fn new() -> Self {
        Self {
            url: PUBLIC_WS_URL.to_string(),
            ping_interval: Duration::from_secs(30),
            on_connect: None,
            on_disconnect: None,
        }
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.ping_interval = interval;
        self
    }

    pub fn on_connect<F>(mut self, cb: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_connect = Some(Box::new(cb));
        self
    }

    pub fn on_disconnect<F>(mut self, cb: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_disconnect = Some(Box::new(cb));
        self
    }

    pub async fn connect(self) -> Result<PublicWsClient> {
        let ws_stream = PublicWsClient::connect_stream_to_url(&self.url).await?;
        let mut interval = tokio::time::interval(self.ping_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;

        let client = PublicWsClient {
            ws_stream,
            subscriptions: HashSet::new(),
            ping_interval: interval,
            ping_interval_duration: self.ping_interval,
            ping_pending: false,
            ws_url: self.url,
            on_connect: self.on_connect,
            on_disconnect: self.on_disconnect,
        };

        if let Some(ref cb) = client.on_connect {
            cb();
        }

        Ok(client)
    }
}

impl PublicWsClient {
    pub fn builder() -> PublicWsClientBuilder {
        PublicWsClientBuilder::new()
    }

    pub fn set_on_connect<F>(&mut self, cb: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_connect = Some(Box::new(cb));
    }

    pub fn set_on_disconnect<F>(&mut self, cb: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_disconnect = Some(Box::new(cb));
    }

    pub async fn connect() -> Result<Self> {
        Self::connect_with_ping_interval(Duration::from_secs(30)).await
    }

    pub async fn connect_with_ping_interval(ping_interval: Duration) -> Result<Self> {
        Self::connect_with_url(PUBLIC_WS_URL, ping_interval).await
    }

    pub async fn connect_with_url(url: &str, ping_interval: Duration) -> Result<Self> {
        let ws_stream = Self::connect_stream_to_url(url).await?;
        let mut interval = tokio::time::interval(ping_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;

        Ok(Self {
            ws_stream,
            subscriptions: HashSet::new(),
            ping_interval: interval,
            ping_interval_duration: ping_interval,
            ping_pending: false,
            ws_url: url.to_string(),
            on_connect: None,
            on_disconnect: None,
        })
    }

    async fn connect_stream_to_url(url_str: &str) -> Result<WsStream> {
        let url = Url::parse(url_str).map_err(|e| GmoFxError::Http(e.to_string()))?;
        let (ws_stream, _) = connect_async(url.as_str())
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
        Ok(ws_stream)
    }

    pub async fn subscribe(&mut self, channel: &str, symbol: Option<&str>) -> Result<()> {
        let mut cmd = SubscribeCommand::new(channel);
        if let Some(sym) = symbol {
            cmd = cmd.symbol(sym);
        }
        let msg = serde_json::to_string(&cmd).map_err(|e| GmoFxError::Json(e.to_string()))?;

        self.ws_stream
            .send(Message::Text(msg.into()))
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;

        self.subscriptions
            .insert((channel.to_string(), symbol.map(String::from)));
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<PublicWsMessage>> {
        loop {
            let msg_fut = self.ws_stream.next();
            let tick_fut = self.ping_interval.tick();

            tokio::select! {
                msg = msg_fut => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            let event: PublicWsMessage =
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
                            // Connection closed, attempt reconnect
                            self.reconnect().await?;
                        }
                        Some(Err(e)) => {
                            eprintln!("WebSocket error: {:?}, attempting reconnect...", e);
                            self.reconnect().await?;
                        }
                        _ => {} // Ignore other message types
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
        if let Some(ref cb) = self.on_disconnect {
            cb();
        }
        let mut attempts = 0;
        loop {
            attempts += 1;
            let backoff = if self.ws_url.contains("127.0.0.1") || self.ws_url.contains("localhost")
            {
                Duration::from_millis(10)
            } else {
                Duration::from_secs(std::cmp::min(2u64.pow(attempts), 60))
            };
            println!("Attempting to reconnect in {:?}...", backoff);
            sleep(backoff).await;

            match Self::connect_stream_to_url(&self.ws_url).await {
                Ok(stream) => {
                    self.ws_stream = stream;
                    self.ping_pending = false;
                    self.ping_interval = tokio::time::interval(self.ping_interval_duration);
                    self.ping_interval.tick().await;
                    println!("Reconnected successfully.");
                    // Resubscribe
                    let subs = self.subscriptions.clone();
                    for (channel, symbol) in subs {
                        let mut cmd = SubscribeCommand::new(&channel);
                        if let Some(sym) = &symbol {
                            cmd = cmd.symbol(sym);
                        }
                        if let Ok(msg) = serde_json::to_string(&cmd) {
                            let _ = self.ws_stream.send(Message::Text(msg.into())).await;
                        }
                    }
                    if let Some(ref cb) = self.on_connect {
                        cb();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::Mutex;
    use tokio_tungstenite::accept_async;

    #[tokio::test]
    async fn test_public_ws_liveness() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        let ping_received = Arc::new(Mutex::new(false));
        let ping_received_clone = ping_received.clone();

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                while let Some(msg) = ws_stream.next().await {
                    match msg.unwrap() {
                        Message::Ping(data) => {
                            *ping_received_clone.lock().await = true;
                            ws_stream.send(Message::Pong(data)).await.unwrap();

                            // Send ticker event AFTER receiving the Ping
                            let ticker_json = r#"{"symbol":"USD_JPY","ask":"157.266","bid":"157.261","timestamp":"2026-05-01T06:06:33.584446Z","status":"OPEN"}"#;
                            ws_stream
                                .send(Message::Text(ticker_json.into()))
                                .await
                                .unwrap();
                        }
                        Message::Text(_) => {
                            // client sent subscribe command
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
        });

        // Use a ping interval of 200ms to prevent race conditions on slow build systems.
        let mut client = PublicWsClient::connect_with_url(&url, Duration::from_millis(200))
            .await
            .unwrap();
        client.subscribe("ticker", Some("USD_JPY")).await.unwrap();

        // next_message() should receive the ticker event after ping-pong is handled
        let msg = client.next_message().await.unwrap();
        assert!(msg.is_some());
        if let Some(PublicWsMessage::Ticker(t)) = msg {
            assert_eq!(t.symbol, "USD_JPY");
        } else {
            panic!("Expected Ticker event");
        }

        // Verify that a Ping was indeed sent and processed
        assert!(*ping_received.lock().await);
        // And ping_pending should be false since we received the Pong
        assert!(!client.ping_pending);
    }

    #[tokio::test]
    async fn test_public_ws_timeout_reconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        let connections = Arc::new(Mutex::new(0));
        let connections_clone = connections.clone();

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let connections_clone = connections_clone.clone();
                tokio::spawn(async move {
                    let conn_idx = {
                        let mut conns = connections_clone.lock().await;
                        *conns += 1;
                        *conns
                    };
                    if let Ok(mut ws_stream) = accept_async(stream).await {
                        if conn_idx == 1 {
                            // Do not read from the stream to prevent tungstenite's automatic Pong response.
                            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                        } else {
                            while let Some(msg) = ws_stream.next().await {
                                if let Message::Text(_) = msg.unwrap() {
                                    // When we get subscription on the new connection, send the ticker event to finish
                                    let ticker_json = r#"{"symbol":"USD_JPY","ask":"157.266","bid":"157.261","timestamp":"2026-05-01T06:06:33.584446Z","status":"OPEN"}"#;
                                    let _ = ws_stream.send(Message::Text(ticker_json.into())).await;
                                }
                            }
                        }
                    }
                });
            }
        });

        // Use a ping interval of 200ms.
        let mut client = PublicWsClient::connect_with_url(&url, Duration::from_millis(200))
            .await
            .unwrap();
        client.subscribe("ticker", Some("USD_JPY")).await.unwrap();

        // next_message() should trigger timeout, reconnect, resubscribe, and receive the ticker event
        let msg = client.next_message().await.unwrap();
        assert!(msg.is_some());
        if let Some(PublicWsMessage::Ticker(t)) = msg {
            assert_eq!(t.symbol, "USD_JPY");
        } else {
            panic!("Expected Ticker event");
        }

        // Verify that at least 2 connections were made (original + reconnect)
        assert!(*connections.lock().await >= 2);
    }
}
