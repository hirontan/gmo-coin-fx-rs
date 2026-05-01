use futures_util::{SinkExt, StreamExt};
use gmo_coin_fx_core::{models::ws::SubscribeCommand, GmoFxError, Result};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub const PUBLIC_WS_URL: &str = "wss://forex-api.coin.z.com/ws/public/v1";

type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub struct PublicWsClient {
    ws_stream: WsStream,
}

impl PublicWsClient {
    pub async fn connect() -> Result<Self> {
        let url = Url::parse(PUBLIC_WS_URL).map_err(|e| GmoFxError::Http(e.to_string()))?;
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| GmoFxError::Http(e.to_string()))?;
        
        Ok(Self { ws_stream })
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
