use serde::{Deserialize, Serialize};
use crate::models::FxSymbol;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    #[serde(rename = "ticker")]
    Ticker,
    #[serde(rename = "orderbooks")]
    Orderbooks,
    #[serde(rename = "executionEvents")]
    ExecutionEvents,
    #[serde(rename = "positionEvents")]
    PositionEvents,
    #[serde(rename = "orderEvents")]
    OrderEvents,
}

impl Channel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ticker => "ticker",
            Self::Orderbooks => "orderbooks",
            Self::ExecutionEvents => "executionEvents",
            Self::PositionEvents => "positionEvents",
            Self::OrderEvents => "orderEvents",
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Channel {
    type Err = crate::error::GmoFxError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "ticker" => Ok(Self::Ticker),
            "orderbooks" => Ok(Self::Orderbooks),
            "executionEvents" => Ok(Self::ExecutionEvents),
            "positionEvents" => Ok(Self::PositionEvents),
            "orderEvents" => Ok(Self::OrderEvents),
            _ => Err(crate::error::GmoFxError::InvalidRequest(format!("unknown channel: {}", s))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subscription {
    pub channel: Channel,
    pub symbol: Option<FxSymbol>,
}

impl Subscription {
    pub fn builder() -> SubscriptionBuilder {
        SubscriptionBuilder::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubscriptionBuilder {
    channel: Option<Channel>,
    symbol: Option<FxSymbol>,
}

impl SubscriptionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn symbol(mut self, symbol: FxSymbol) -> Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn symbol_opt(mut self, symbol: Option<FxSymbol>) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn build(self) -> Subscription {
        Subscription {
            channel: self.channel.expect("channel is required"),
            symbol: self.symbol,
        }
    }
}
