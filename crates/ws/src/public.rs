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
}

impl PublicWsClient {
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
        let mut attempts = 0;
        loop {
            attempts += 1;
            let backoff = std::cmp::min(2u64.pow(attempts), 60);
            println!("Attempting to reconnect in {} seconds...", backoff);
            sleep(Duration::from_secs(backoff)).await;

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
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Reconnect failed: {:?}", e);
                }
            }
        }
    }
}
