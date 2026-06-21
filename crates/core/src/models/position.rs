use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    #[serde(rename = "positionId")]
    pub position_id: u64,

    pub symbol: String,
    pub side: String,
    pub size: String,

    #[serde(rename = "orderdSize")]
    pub ordered_size: String,

    pub price: String,

    #[serde(rename = "lossGain")]
    pub loss_gain: String,

    pub leverage: String,

    #[serde(rename = "losscutPrice")]
    pub losscut_price: String,

    pub timestamp: String,
}

impl Position {
    pub fn size_f64(&self) -> crate::Result<f64> {
        self.size.parse::<f64>().map_err(Into::into)
    }

    pub fn ordered_size_f64(&self) -> crate::Result<f64> {
        self.ordered_size.parse::<f64>().map_err(Into::into)
    }

    pub fn price_f64(&self) -> crate::Result<f64> {
        self.price.parse::<f64>().map_err(Into::into)
    }

    pub fn loss_gain_f64(&self) -> crate::Result<f64> {
        self.loss_gain.parse::<f64>().map_err(Into::into)
    }

    pub fn leverage_f64(&self) -> crate::Result<f64> {
        self.leverage.parse::<f64>().map_err(Into::into)
    }

    pub fn losscut_price_f64(&self) -> crate::Result<f64> {
        self.losscut_price.parse::<f64>().map_err(Into::into)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionsList {
    pub list: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSummary {
    pub symbol: String,
    pub side: String,

    #[serde(rename = "averagePositionRate")]
    pub average_position_rate: String,

    #[serde(rename = "positionLossGain")]
    pub position_loss_gain: String,

    #[serde(rename = "sumOrderQuantity")]
    pub sum_order_quantity: String,

    #[serde(rename = "sumPositionQuantity")]
    pub sum_position_quantity: String,
}

impl PositionSummary {
    pub fn average_position_rate_f64(&self) -> crate::Result<f64> {
        self.average_position_rate
            .parse::<f64>()
            .map_err(Into::into)
    }

    pub fn position_loss_gain_f64(&self) -> crate::Result<f64> {
        self.position_loss_gain.parse::<f64>().map_err(Into::into)
    }

    pub fn sum_order_quantity_f64(&self) -> crate::Result<f64> {
        self.sum_order_quantity.parse::<f64>().map_err(Into::into)
    }

    pub fn sum_position_quantity_f64(&self) -> crate::Result<f64> {
        self.sum_position_quantity
            .parse::<f64>()
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSummaryList {
    pub list: Vec<PositionSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_f64() {
        let pos = Position {
            position_id: 123,
            symbol: "USD_JPY".to_string(),
            side: "BUY".to_string(),
            size: "10000".to_string(),
            ordered_size: "1000".to_string(),
            price: "155.25".to_string(),
            loss_gain: "-500".to_string(),
            leverage: "25".to_string(),
            losscut_price: "154.0".to_string(),
            timestamp: "now".to_string(),
        };

        assert_eq!(pos.size_f64().unwrap(), 10000.0);
        assert_eq!(pos.ordered_size_f64().unwrap(), 1000.0);
        assert_eq!(pos.price_f64().unwrap(), 155.25);
        assert_eq!(pos.loss_gain_f64().unwrap(), -500.0);
        assert_eq!(pos.leverage_f64().unwrap(), 25.0);
        assert_eq!(pos.losscut_price_f64().unwrap(), 154.0);
    }

    #[test]
    fn test_position_summary_f64() {
        let summary = PositionSummary {
            symbol: "USD_JPY".to_string(),
            side: "BUY".to_string(),
            average_position_rate: "155.25".to_string(),
            position_loss_gain: "-500".to_string(),
            sum_order_quantity: "1000".to_string(),
            sum_position_quantity: "10000".to_string(),
        };

        assert_eq!(summary.average_position_rate_f64().unwrap(), 155.25);
        assert_eq!(summary.position_loss_gain_f64().unwrap(), -500.0);
        assert_eq!(summary.sum_order_quantity_f64().unwrap(), 1000.0);
        assert_eq!(summary.sum_position_quantity_f64().unwrap(), 10000.0);
    }

    #[test]
    fn test_deserialize_position() {
        let json = r#"
        {
            "positionId": 1234567,
            "symbol": "USD_JPY",
            "side": "BUY",
            "size": "10000",
            "orderdSize": "0",
            "price": "135.5",
            "lossGain": "1000",
            "leverage": "25",
            "losscutPrice": "130.0",
            "timestamp": "2019-03-19T02:15:06.064Z"
        }
        "#;
        let pos: Position = serde_json::from_str(json).unwrap();
        assert_eq!(pos.position_id, 1234567);
        assert_eq!(pos.symbol, "USD_JPY");
    }
}
