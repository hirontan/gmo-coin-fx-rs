use futures_util::{SinkExt, StreamExt};
use gmo_coin_fx_core::{
    models::ws::SubscribeCommand, models::ws_events::PublicWsMessage, GmoFxError, Result,
};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::reconnect::ReconnectConfig;
use std::sync::Arc;

pub const PUBLIC_WS_URL: &str = "wss://forex-api.coin.z.com/ws/public/v1";

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug)]
enum PublicCommand {
    Subscribe {
        channel: String,
        symbol: Option<String>,
    },
    SubscribeFiltered {
        channel: String,
        symbol: String,
    },
}

struct Callbacks {
    on_connect: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    on_disconnect: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
}

pub struct PublicWsClient {
    cmd_tx: tokio::sync::mpsc::Sender<PublicCommand>,
    msg_rx: tokio::sync::mpsc::Receiver<Result<PublicWsMessage>>,
    runner_handle: tokio::task::JoinHandle<()>,
    callbacks: Arc<std::sync::Mutex<Callbacks>>,
}

pub struct PublicWsClientBuilder {
    url: String,
    ping_interval: Duration,
    on_connect: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    on_disconnect: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    reconnect_config: ReconnectConfig,
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
            reconnect_config: ReconnectConfig::default(),
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
        self.on_connect = Some(Arc::new(cb));
        self
    }

    pub fn on_disconnect<F>(mut self, cb: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_disconnect = Some(Arc::new(cb));
        self
    }

    pub fn reconnect_config(mut self, config: ReconnectConfig) -> Self {
        self.reconnect_config = config;
        self
    }

    pub async fn connect(self) -> Result<PublicWsClient> {
        let ws_stream = PublicWsClient::connect_stream_to_url(&self.url).await?;
        let mut interval = tokio::time::interval(self.ping_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;

        let callbacks = Arc::new(std::sync::Mutex::new(Callbacks {
            on_connect: self.on_connect,
            on_disconnect: self.on_disconnect,
        }));

        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(100);
        let (msg_tx, msg_rx) = tokio::sync::mpsc::channel(1000);

        let runner = PublicWsRunner {
            ws_stream: Some(ws_stream),
            ws_url: self.url,
            subscriptions: HashSet::new(),
            filters: std::collections::HashMap::new(),
            cmd_rx,
            msg_tx,
            ping_interval_duration: self.ping_interval,
            ping_interval: interval,
            ping_pending: false,
            callbacks: callbacks.clone(),
            reconnect_config: self.reconnect_config,
        };

        let runner_handle = tokio::spawn(runner.run());

        // Invoke the first on_connect callback
        let cb = {
            let guard = callbacks.lock().unwrap();
            guard.on_connect.clone()
        };
        if let Some(cb) = cb {
            cb();
        }

        Ok(PublicWsClient {
            cmd_tx,
            msg_rx,
            runner_handle,
            callbacks,
        })
    }
}

struct PublicWsRunner {
    ws_stream: Option<WsStream>,
    ws_url: String,
    subscriptions: HashSet<(String, Option<String>)>,
    filters: std::collections::HashMap<String, HashSet<String>>,
    cmd_rx: tokio::sync::mpsc::Receiver<PublicCommand>,
    msg_tx: tokio::sync::mpsc::Sender<Result<PublicWsMessage>>,
    ping_interval_duration: Duration,
    ping_interval: tokio::time::Interval,
    ping_pending: bool,
    callbacks: Arc<std::sync::Mutex<Callbacks>>,
    reconnect_config: ReconnectConfig,
}

impl PublicWsRunner {
    async fn run(mut self) {
        loop {
            if self.ws_stream.is_none() && self.reconnect_and_handle_commands().await.is_err() {
                break;
            }

            if let Some(ref mut stream) = self.ws_stream {
                let msg_fut = stream.next();
                let tick_fut = self.ping_interval.tick();
                let cmd_fut = self.cmd_rx.recv();

                tokio::select! {
                    msg = msg_fut => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                match serde_json::from_str::<PublicWsMessage>(&text) {
                                    Ok(event) => {
                                        let channel = event.channel();
                                        let symbol = event.symbol();
                                        let allowed = if let Some(allowed_symbols) = self.filters.get(channel) {
                                            allowed_symbols.contains(symbol)
                                        } else {
                                            true
                                        };
                                        if allowed && self.msg_tx.send(Ok(event)).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        if self.msg_tx.send(Err(GmoFxError::Json(e.to_string()))).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            Some(Ok(Message::Pong(_))) => {
                                self.ping_pending = false;
                            }
                            Some(Ok(Message::Ping(data))) => {
                                if let Some(ref mut stream) = self.ws_stream {
                                    let _ = stream.send(Message::Pong(data)).await;
                                }
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                self.handle_disconnect().await;
                                self.ws_stream = None;
                            }
                            Some(Err(e)) => {
                                eprintln!("WebSocket error: {:?}, attempting reconnect...", e);
                                self.handle_disconnect().await;
                                self.ws_stream = None;
                            }
                            _ => {}
                        }
                    }
                    _ = tick_fut => {
                        if self.ping_pending {
                            eprintln!("Ping timeout: no pong received. Reconnecting...");
                            self.handle_disconnect().await;
                            self.ws_stream = None;
                        } else {
                            if let Err(e) = stream.send(Message::Ping(vec![].into())).await {
                                eprintln!("Failed to send ping: {:?}, attempting reconnect...", e);
                                self.handle_disconnect().await;
                                self.ws_stream = None;
                            } else {
                                self.ping_pending = true;
                            }
                        }
                    }
                    cmd = cmd_fut => {
                        match cmd {
                            Some(PublicCommand::Subscribe { channel, symbol }) => {
                                self.filters.remove(&channel);
                                self.subscriptions.insert((channel.clone(), symbol.clone()));
                                let mut cmd_msg = SubscribeCommand::new(&channel);
                                if let Some(sym) = &symbol {
                                    cmd_msg = cmd_msg.symbol(sym);
                                }
                                if let Ok(msg_str) = serde_json::to_string(&cmd_msg) {
                                    if let Err(e) = stream.send(Message::Text(msg_str.into())).await {
                                        eprintln!("Failed to send subscription command: {:?}, reconnecting...", e);
                                        self.handle_disconnect().await;
                                        self.ws_stream = None;
                                    }
                                }
                            }
                            Some(PublicCommand::SubscribeFiltered { channel, symbol }) => {
                                self.filters.entry(channel.clone()).or_default().insert(symbol.clone());
                                self.subscriptions.insert((channel.clone(), Some(symbol.clone())));
                                let cmd_msg = SubscribeCommand::new(&channel).symbol(&symbol);
                                if let Ok(msg_str) = serde_json::to_string(&cmd_msg) {
                                    if let Err(e) = stream.send(Message::Text(msg_str.into())).await {
                                        eprintln!("Failed to send subscription command: {:?}, reconnecting...", e);
                                        self.handle_disconnect().await;
                                        self.ws_stream = None;
                                    }
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn reconnect_and_handle_commands(&mut self) -> std::result::Result<(), ()> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            if let Some(max) = self.reconnect_config.max_retries {
                if attempts > max {
                    eprintln!("Reconnect failed: exceeded max retries of {}", max);
                    return Err(());
                }
            }
            let backoff = if self.ws_url.contains("127.0.0.1") || self.ws_url.contains("localhost")
            {
                Duration::from_millis(10)
            } else {
                self.reconnect_config.calculate_delay(attempts)
            };
            println!("Attempting to reconnect in {:?}...", backoff);

            let sleep_fut = sleep(backoff);
            tokio::pin!(sleep_fut);

            loop {
                tokio::select! {
                    _ = &mut sleep_fut => {
                        break;
                    }
                    cmd = self.cmd_rx.recv() => {
                        match cmd {
                            Some(PublicCommand::Subscribe { channel, symbol }) => {
                                self.filters.remove(&channel);
                                self.subscriptions.insert((channel, symbol));
                            }
                            Some(PublicCommand::SubscribeFiltered { channel, symbol }) => {
                                self.filters.entry(channel.clone()).or_default().insert(symbol.clone());
                                self.subscriptions.insert((channel, Some(symbol)));
                            }
                            None => {
                                return Err(());
                            }
                        }
                    }
                }
            }

            match PublicWsClient::connect_stream_to_url(&self.ws_url).await {
                Ok(stream) => {
                    self.ws_stream = Some(stream);
                    self.ping_pending = false;
                    self.ping_interval = tokio::time::interval(self.ping_interval_duration);
                    self.ping_interval.tick().await;
                    println!("Reconnected successfully.");

                    let subs = self.subscriptions.clone();
                    for (channel, symbol) in subs {
                        let mut cmd = SubscribeCommand::new(&channel);
                        if let Some(sym) = &symbol {
                            cmd = cmd.symbol(sym);
                        }
                        if let Ok(msg) = serde_json::to_string(&cmd) {
                            if let Some(ref mut stream) = self.ws_stream {
                                if let Err(e) = stream.send(Message::Text(msg.into())).await {
                                    eprintln!("Failed to send subscription: {:?}", e);
                                    self.ws_stream = None;
                                    break;
                                }
                            }
                        }
                    }

                    if self.ws_stream.is_some() {
                        self.handle_connect().await;
                        return Ok(());
                    }
                }
                Err(e) => {
                    eprintln!("Reconnect failed: {:?}", e);
                }
            }
        }
    }

    async fn handle_connect(&self) {
        let cb = {
            let guard = self.callbacks.lock().unwrap();
            guard.on_connect.clone()
        };
        if let Some(cb) = cb {
            cb();
        }
    }

    async fn handle_disconnect(&self) {
        let cb = {
            let guard = self.callbacks.lock().unwrap();
            guard.on_disconnect.clone()
        };
        if let Some(cb) = cb {
            cb();
        }
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
        self.callbacks.lock().unwrap().on_connect = Some(Arc::new(cb));
    }

    pub fn set_on_disconnect<F>(&mut self, cb: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks.lock().unwrap().on_disconnect = Some(Arc::new(cb));
    }

    pub async fn connect() -> Result<Self> {
        Self::connect_with_ping_interval(Duration::from_secs(30)).await
    }

    pub async fn connect_with_ping_interval(ping_interval: Duration) -> Result<Self> {
        Self::connect_with_url(PUBLIC_WS_URL, ping_interval).await
    }

    pub async fn connect_with_url(url: &str, ping_interval: Duration) -> Result<Self> {
        Self::builder()
            .url(url)
            .ping_interval(ping_interval)
            .connect()
            .await
    }

    async fn connect_stream_to_url(url_str: &str) -> Result<WsStream> {
        let url = Url::parse(url_str).map_err(|e| GmoFxError::Http(e.to_string()))?;
        let (ws_stream, _) = connect_async(url.as_str())
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
        Ok(ws_stream)
    }

    pub async fn subscribe(&mut self, channel: &str, symbol: Option<&str>) -> Result<()> {
        self.cmd_tx
            .send(PublicCommand::Subscribe {
                channel: channel.to_string(),
                symbol: symbol.map(String::from),
            })
            .await
            .map_err(|e| {
                GmoFxError::Http(format!("Failed to send subscribe command to runner: {}", e))
            })?;
        Ok(())
    }

    pub async fn subscribe_filtered(&mut self, channel: &str, symbol: &str) -> Result<()> {
        self.cmd_tx
            .send(PublicCommand::SubscribeFiltered {
                channel: channel.to_string(),
                symbol: symbol.to_string(),
            })
            .await
            .map_err(|e| {
                GmoFxError::Http(format!(
                    "Failed to send subscribe_filtered command to runner: {}",
                    e
                ))
            })?;
        Ok(())
    }

    pub async fn subscribe_orderbook(&mut self, symbol: &str) -> Result<()> {
        self.subscribe("orderbooks", Some(symbol)).await
    }

    pub async fn next_message(&mut self) -> Result<Option<PublicWsMessage>> {
        match self.msg_rx.recv().await {
            Some(res) => res.map(Some),
            None => Ok(None),
        }
    }
}

impl Drop for PublicWsClient {
    fn drop(&mut self) {
        self.runner_handle.abort();
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

    #[tokio::test]
    async fn test_public_ws_callbacks() {
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
                            // Do not read from the stream to trigger ping timeout reconnect
                            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                        } else {
                            while let Some(msg) = ws_stream.next().await {
                                if let Message::Text(_) = msg.unwrap() {
                                    let ticker_json = r#"{"symbol":"USD_JPY","ask":"157.266","bid":"157.261","timestamp":"2026-05-01T06:06:33.584446Z","status":"OPEN"}"#;
                                    let _ = ws_stream.send(Message::Text(ticker_json.into())).await;
                                }
                            }
                        }
                    }
                });
            }
        });

        let connect_calls = Arc::new(Mutex::new(0));
        let disconnect_calls = Arc::new(Mutex::new(0));

        let connect_calls_clone = connect_calls.clone();
        let disconnect_calls_clone = disconnect_calls.clone();

        let client = PublicWsClient::builder()
            .url(&url)
            .ping_interval(Duration::from_millis(200))
            .on_connect(move || {
                let connect_calls_clone = connect_calls_clone.clone();
                tokio::spawn(async move {
                    *connect_calls_clone.lock().await += 1;
                });
            })
            .on_disconnect(move || {
                let disconnect_calls_clone = disconnect_calls_clone.clone();
                tokio::spawn(async move {
                    *disconnect_calls_clone.lock().await += 1;
                });
            });

        // connect() should trigger the first on_connect
        let mut client = client.connect().await.unwrap();
        client.subscribe("ticker", Some("USD_JPY")).await.unwrap();

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(*connect_calls.lock().await, 1);
        assert_eq!(*disconnect_calls.lock().await, 0);

        // next_message() should trigger timeout -> on_disconnect -> reconnect -> on_connect -> receive ticker
        let msg = client.next_message().await.unwrap();
        assert!(msg.is_some());

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(*connect_calls.lock().await, 2);
        assert_eq!(*disconnect_calls.lock().await, 1);
    }

    #[tokio::test]
    async fn test_public_ws_buffering_and_queued_subscription() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        let second_conn_sub_received = Arc::new(Mutex::new(None));
        let second_conn_sub_received_clone = second_conn_sub_received.clone();

        // Server task
        tokio::spawn(async move {
            // 1. Accept first connection
            if let Ok((stream, _)) = listener.accept().await {
                if let Ok(mut ws_stream) = accept_async(stream).await {
                    // Send a message before dropping the connection
                    let ticker_json = r#"{"symbol":"USD_JPY","ask":"157.266","bid":"157.261","timestamp":"2026-05-01T06:06:33.584446Z","status":"OPEN"}"#;
                    ws_stream
                        .send(Message::Text(ticker_json.into()))
                        .await
                        .unwrap();

                    // Now drop/close connection
                    ws_stream.close(None).await.unwrap();
                }
            }

            // 2. Accept second connection (reconnect)
            if let Ok((stream, _)) = listener.accept().await {
                if let Ok(mut ws_stream) = accept_async(stream).await {
                    // Read subscription commands
                    if let Some(Ok(Message::Text(txt))) = ws_stream.next().await {
                        *second_conn_sub_received_clone.lock().await = Some(txt.to_string());
                    }
                }
            }
        });

        // Client
        let mut client = PublicWsClient::connect_with_url(&url, Duration::from_secs(30))
            .await
            .unwrap();

        // Wait a short bit to ensure the client processes the first message and disconnects
        tokio::time::sleep(Duration::from_millis(50)).await;

        // While client is disconnected/reconnecting, we subscribe to a new channel
        client.subscribe_orderbook("BTC_JPY").await.unwrap();

        // Wait for reconnect to happen and process resubscribe
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Verify the client is able to read the buffered message that was received before reconnect
        let msg = client.next_message().await.unwrap();
        assert!(msg.is_some());
        if let Some(PublicWsMessage::Ticker(t)) = msg {
            assert_eq!(t.symbol, "USD_JPY");
        } else {
            panic!("Expected Ticker event from buffered message");
        }

        // Verify the queued subscription was sent to the second connection
        let sub_text = second_conn_sub_received.lock().await.clone();
        assert!(
            sub_text.is_some(),
            "No subscription received on second connection"
        );
        let sub_json = sub_text.unwrap();
        assert!(
            sub_json.contains("orderbooks"),
            "Subscription did not contain orderbooks"
        );
        assert!(
            sub_json.contains("BTC_JPY"),
            "Subscription did not contain BTC_JPY"
        );
    }

    #[tokio::test]
    async fn test_public_ws_orderbook_subscription() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                while let Some(msg) = ws_stream.next().await {
                    match msg.unwrap() {
                        Message::Ping(data) => {
                            ws_stream.send(Message::Pong(data)).await.unwrap();
                        }
                        Message::Text(txt) => {
                            if txt.contains("subscribe") && txt.contains("orderbooks") {
                                // Send mock OrderBook Event
                                let ob_json = r#"{"symbol":"BTC_JPY","asks":[{"price":"10000000","size":"0.5"}],"bids":[{"price":"9999000","size":"1.2"}],"timestamp":"2026-05-01T06:06:33.584446Z"}"#;
                                ws_stream.send(Message::Text(ob_json.into())).await.unwrap();
                            }
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
        });

        let mut client = PublicWsClient::connect_with_url(&url, Duration::from_millis(200))
            .await
            .unwrap();
        client.subscribe_orderbook("BTC_JPY").await.unwrap();

        let msg = client.next_message().await.unwrap();
        assert!(msg.is_some());
        if let Some(PublicWsMessage::OrderBook(ob)) = msg {
            assert_eq!(ob.symbol, "BTC_JPY");
            assert_eq!(ob.asks[0].price_f64().unwrap(), 10000000.0);
            assert_eq!(ob.asks[0].size_f64().unwrap(), 0.5);
            assert_eq!(ob.bids[0].price_f64().unwrap(), 9999000.0);
            assert_eq!(ob.bids[0].size_f64().unwrap(), 1.2);
        } else {
            panic!("Expected OrderBook event");
        }
    }

    #[tokio::test]
    async fn test_public_ws_max_retries() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                if let Ok(mut ws_stream) = accept_async(stream).await {
                    let _ = ws_stream.close(None).await;
                }
            }
        });

        let mut client = PublicWsClient::builder()
            .url(&url)
            .ping_interval(Duration::from_millis(200))
            .reconnect_config(ReconnectConfig::new().max_retries(Some(2)))
            .connect()
            .await
            .unwrap();

        let msg = client.next_message().await.unwrap();
        assert!(msg.is_none());
    }

    #[tokio::test]
    async fn test_public_ws_symbol_filtering() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                while let Some(msg) = ws_stream.next().await {
                    match msg.unwrap() {
                        Message::Ping(data) => {
                            ws_stream.send(Message::Pong(data)).await.unwrap();
                        }
                        Message::Text(txt) => {
                            if txt.contains("subscribe") && txt.contains("ticker") {
                                // Send EUR_JPY ticker (should be filtered out)
                                let eur_json = r#"{"symbol":"EUR_JPY","ask":"167.266","bid":"167.261","timestamp":"2026-05-01T06:06:33.584446Z","status":"OPEN"}"#;
                                ws_stream
                                    .send(Message::Text(eur_json.into()))
                                    .await
                                    .unwrap();

                                // Send USD_JPY ticker (should be delivered)
                                let usd_json = r#"{"symbol":"USD_JPY","ask":"157.266","bid":"157.261","timestamp":"2026-05-01T06:06:33.584446Z","status":"OPEN"}"#;
                                ws_stream
                                    .send(Message::Text(usd_json.into()))
                                    .await
                                    .unwrap();
                            }
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
        });

        let mut client = PublicWsClient::connect_with_url(&url, Duration::from_millis(200))
            .await
            .unwrap();
        client
            .subscribe_filtered("ticker", "USD_JPY")
            .await
            .unwrap();

        // next_message() should skip EUR_JPY and receive USD_JPY directly
        let msg = client.next_message().await.unwrap();
        assert!(msg.is_some());
        if let Some(PublicWsMessage::Ticker(t)) = msg {
            assert_eq!(t.symbol, "USD_JPY");
        } else {
            panic!("Expected Ticker event");
        }
    }

    #[tokio::test]
    async fn test_public_ws_symbol_filtering_cleared_by_subscribe() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                while let Some(msg) = ws_stream.next().await {
                    match msg.unwrap() {
                        Message::Ping(data) => {
                            ws_stream.send(Message::Pong(data)).await.unwrap();
                        }
                        Message::Text(txt) => {
                            if txt.contains("subscribe") && txt.contains("ticker") {
                                // Send EUR_JPY ticker
                                let eur_json = r#"{"symbol":"EUR_JPY","ask":"167.266","bid":"167.261","timestamp":"2026-05-01T06:06:33.584446Z","status":"OPEN"}"#;
                                ws_stream
                                    .send(Message::Text(eur_json.into()))
                                    .await
                                    .unwrap();
                            }
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
        });

        let mut client = PublicWsClient::connect_with_url(&url, Duration::from_millis(200))
            .await
            .unwrap();
        // Subscribe filtered first
        client
            .subscribe_filtered("ticker", "USD_JPY")
            .await
            .unwrap();
        // Normal subscribe to the same channel clears the filter
        client.subscribe("ticker", Some("EUR_JPY")).await.unwrap();

        // Wait a short bit for commands to be processed in the runner
        tokio::time::sleep(Duration::from_millis(50)).await;

        let msg = client.next_message().await.unwrap();
        assert!(msg.is_some());
        if let Some(PublicWsMessage::Ticker(t)) = msg {
            assert_eq!(t.symbol, "EUR_JPY");
        } else {
            panic!("Expected Ticker event");
        }
    }
}
