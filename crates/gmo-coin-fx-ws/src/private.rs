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
}

impl PrivateWsClient {
    pub async fn connect(client: GmoFxClient) -> Result<Self> {
        let (ws_stream, renew_task) = Self::connect_stream(&client).await?;

        Ok(Self {
            ws_stream,
            renew_task: Some(renew_task),
            client,
            subscriptions: HashSet::new(),
        })
    }

    async fn connect_stream(
        client: &GmoFxClient,
    ) -> Result<(WsStream, tokio::task::JoinHandle<()>)> {
        let auth = client.ws_auth_post().await?;
        let url_str = format!("{}/{}", PRIVATE_WS_URL, auth.token);
        let url = Url::parse(&url_str).map_err(|e| GmoFxError::Http(e.to_string()))?;

        let (ws_stream, _) = connect_async(url)
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
            .send(Message::Text(msg))
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;

        self.subscriptions.insert(channel.to_string());
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<PrivateWsMessage>> {
        loop {
            match self.ws_stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let event: PrivateWsMessage =
                        serde_json::from_str(&text).map_err(|e| GmoFxError::Json(e.to_string()))?;
                    return Ok(Some(event));
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
    }

    async fn reconnect(&mut self) -> Result<()> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let backoff = std::cmp::min(2u64.pow(attempts), 60);
            println!("Attempting to reconnect in {} seconds...", backoff);
            sleep(Duration::from_secs(backoff)).await;

            if let Some(task) = self.renew_task.take() {
                task.abort();
            }

            match Self::connect_stream(&self.client).await {
                Ok((stream, task)) => {
                    self.ws_stream = stream;
                    self.renew_task = Some(task);
                    println!("Reconnected successfully.");

                    let subs = self.subscriptions.clone();
                    for channel in subs {
                        let cmd = SubscribeCommand::new(&channel);
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

impl Drop for PrivateWsClient {
    fn drop(&mut self) {
        if let Some(task) = self.renew_task.take() {
            task.abort();
        }
    }
}
