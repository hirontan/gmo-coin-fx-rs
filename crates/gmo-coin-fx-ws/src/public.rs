use futures_util::{SinkExt, StreamExt};
use gmo_coin_fx_core::{models::ws::SubscribeCommand, models::ws_events::PublicWsMessage, GmoFxError, Result};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub const PUBLIC_WS_URL: &str = "wss://forex-api.coin.z.com/ws/public/v1";

type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub struct PublicWsClient {
    ws_stream: WsStream,
    subscriptions: HashSet<(String, Option<String>)>,
}

impl PublicWsClient {
    pub async fn connect() -> Result<Self> {
        let ws_stream = Self::connect_stream().await?;
        Ok(Self {
            ws_stream,
            subscriptions: HashSet::new(),
        })
    }

    async fn connect_stream() -> Result<WsStream> {
        let url = Url::parse(PUBLIC_WS_URL).map_err(|e| GmoFxError::Http(e.to_string()))?;
        let (ws_stream, _) = connect_async(url)
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
            .send(Message::Text(msg))
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
        
        self.subscriptions.insert((channel.to_string(), symbol.map(String::from)));
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<PublicWsMessage>> {
        loop {
            match self.ws_stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let event: PublicWsMessage = serde_json::from_str(&text)
                        .map_err(|e| GmoFxError::Json(e.to_string()))?;
                    return Ok(Some(event));
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
    }

    async fn reconnect(&mut self) -> Result<()> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let backoff = std::cmp::min(2u64.pow(attempts), 60);
            println!("Attempting to reconnect in {} seconds...", backoff);
            sleep(Duration::from_secs(backoff)).await;

            match Self::connect_stream().await {
                Ok(stream) => {
                    self.ws_stream = stream;
                    println!("Reconnected successfully.");
                    // Resubscribe
                    let subs = self.subscriptions.clone();
                    for (channel, symbol) in subs {
                        let mut cmd = SubscribeCommand::new(&channel);
                        if let Some(sym) = &symbol {
                            cmd = cmd.symbol(sym);
                        }
                        if let Ok(msg) = serde_json::to_string(&cmd) {
                            let _ = self.ws_stream.send(Message::Text(msg)).await;
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
