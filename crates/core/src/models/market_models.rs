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

impl Ticker {
    pub fn ask_f64(&self) -> crate::Result<f64> {
        self.ask.parse::<f64>().map_err(Into::into)
    }

    pub fn bid_f64(&self) -> crate::Result<f64> {
        self.bid.parse::<f64>().map_err(Into::into)
    }
}

impl Kline {
    pub fn open_f64(&self) -> crate::Result<f64> {
        self.open.parse::<f64>().map_err(Into::into)
    }

    pub fn high_f64(&self) -> crate::Result<f64> {
        self.high.parse::<f64>().map_err(Into::into)
    }

    pub fn low_f64(&self) -> crate::Result<f64> {
        self.low.parse::<f64>().map_err(Into::into)
    }

    pub fn close_f64(&self) -> crate::Result<f64> {
        self.close.parse::<f64>().map_err(Into::into)
    }
}

impl Symbol {
    pub fn min_open_order_size_f64(&self) -> crate::Result<f64> {
        self.min_open_order_size.parse::<f64>().map_err(Into::into)
    }

    pub fn max_order_size_f64(&self) -> crate::Result<f64> {
        self.max_order_size.parse::<f64>().map_err(Into::into)
    }

    pub fn size_step_f64(&self) -> crate::Result<f64> {
        self.size_step.parse::<f64>().map_err(Into::into)
    }

    pub fn tick_size_f64(&self) -> crate::Result<f64> {
        self.tick_size.parse::<f64>().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticker_f64() {
        let ticker = Ticker {
            symbol: "USD_JPY".to_string(),
            ask: "155.25".to_string(),
            bid: "155.20".to_string(),
            timestamp: "now".to_string(),
            status: "OPEN".to_string(),
        };
        assert_eq!(ticker.ask_f64().unwrap(), 155.25);
        assert_eq!(ticker.bid_f64().unwrap(), 155.20);

        let invalid = Ticker {
            symbol: "USD_JPY".to_string(),
            ask: "invalid".to_string(),
            bid: "155.20".to_string(),
            timestamp: "now".to_string(),
            status: "OPEN".to_string(),
        };
        assert!(invalid.ask_f64().is_err());
    }

    #[test]
    fn test_kline_f64() {
        let kline = Kline {
            open_time: "10:00".to_string(),
            open: "155.1".to_string(),
            high: "155.5".to_string(),
            low: "155.0".to_string(),
            close: "155.3".to_string(),
        };
        assert_eq!(kline.open_f64().unwrap(), 155.1);
        assert_eq!(kline.high_f64().unwrap(), 155.5);
        assert_eq!(kline.low_f64().unwrap(), 155.0);
        assert_eq!(kline.close_f64().unwrap(), 155.3);
    }

    #[test]
    fn test_symbol_f64() {
        let sym = Symbol {
            symbol: "USD_JPY".to_string(),
            min_open_order_size: "0.1".to_string(),
            max_order_size: "100".to_string(),
            size_step: "0.01".to_string(),
            tick_size: "0.001".to_string(),
        };
        assert_eq!(sym.min_open_order_size_f64().unwrap(), 0.1);
        assert_eq!(sym.max_order_size_f64().unwrap(), 100.0);
        assert_eq!(sym.size_step_f64().unwrap(), 0.01);
        assert_eq!(sym.tick_size_f64().unwrap(), 0.001);
    }
}

