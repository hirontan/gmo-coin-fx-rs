use futures_util::{SinkExt, StreamExt};
use gmo_coin_fx_client::GmoFxClient;
use gmo_coin_fx_core::{models::ws::SubscribeCommand, GmoFxError, Result};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub const PRIVATE_WS_URL: &str = "wss://forex-api.coin.z.com/ws/private/v1";

type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub struct PrivateWsClient {
    ws_stream: WsStream,
    renew_task: Option<tokio::task::JoinHandle<()>>,
}

impl PrivateWsClient {
    pub async fn connect(client: GmoFxClient) -> Result<Self> {
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

        Ok(Self {
            ws_stream,
            renew_task: Some(renew_task),
        })
    }

    pub async fn subscribe(&mut self, channel: &str) -> Result<()> {
        let cmd = SubscribeCommand::new(channel);
        let msg = serde_json::to_string(&cmd).map_err(|e| GmoFxError::Json(e.to_string()))?;

        self.ws_stream
            .send(Message::Text(msg))
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;

        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<String>> {
        while let Some(msg) = self.ws_stream.next().await {
            let msg = msg.map_err(|e| GmoFxError::Http(e.to_string()))?;
            if let Message::Text(text) = msg {
                return Ok(Some(text));
            }
        }
        Ok(None)
    }
}

impl Drop for PrivateWsClient {
    fn drop(&mut self) {
        if let Some(task) = self.renew_task.take() {
            task.abort();
        }
    }
}
