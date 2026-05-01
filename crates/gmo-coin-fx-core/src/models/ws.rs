use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct WsAuth {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscribeCommand {
    pub command: String,
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,
}

impl SubscribeCommand {
    pub fn new(channel: impl Into<String>) -> Self {
        Self {
            command: "subscribe".to_string(),
            channel: channel.into(),
            symbol: None,
            option: None,
        }
    }

    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }
}
