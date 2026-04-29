use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub ask: String,
    pub bid: String,
    pub timestamp: String,
    pub status: String,
}
