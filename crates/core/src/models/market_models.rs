use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub ask: String,
    pub bid: String,
    pub timestamp: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiStatus {
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Kline {
    #[serde(rename = "openTime")]
    pub open_time: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Symbol {
    pub symbol: String,
    #[serde(rename = "minOpenOrderSize")]
    pub min_open_order_size: String,
    #[serde(rename = "maxOrderSize")]
    pub max_order_size: String,
    #[serde(rename = "sizeStep")]
    pub size_step: String,
    #[serde(rename = "tickSize")]
    pub tick_size: String,
}
