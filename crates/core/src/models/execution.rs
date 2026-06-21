use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    #[serde(rename = "executionId")]
    pub execution_id: u64,

    #[serde(rename = "orderId")]
    pub order_id: u64,

    pub symbol: String,
    pub side: String,

    #[serde(rename = "settleType")]
    pub settle_type: String,

    pub size: String,
    pub price: String,

    #[serde(rename = "lossGain")]
    pub loss_gain: String,

    pub fee: String,
    pub timestamp: String,
}

impl Execution {
    pub fn size_f64(&self) -> crate::Result<f64> {
        self.size.parse::<f64>().map_err(Into::into)
    }

    pub fn price_f64(&self) -> crate::Result<f64> {
        self.price.parse::<f64>().map_err(Into::into)
    }

    pub fn loss_gain_f64(&self) -> crate::Result<f64> {
        self.loss_gain.parse::<f64>().map_err(Into::into)
    }

    pub fn fee_f64(&self) -> crate::Result<f64> {
        self.fee.parse::<f64>().map_err(Into::into)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionsList {
    pub list: Vec<Execution>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_f64() {
        let exec = Execution {
            execution_id: 12345,
            order_id: 67890,
            symbol: "USD_JPY".to_string(),
            side: "BUY".to_string(),
            settle_type: "OPEN".to_string(),
            size: "10000.5".to_string(),
            price: "155.25".to_string(),
            loss_gain: "1234.5".to_string(),
            fee: "0.0".to_string(),
            timestamp: "now".to_string(),
        };

        assert_eq!(exec.size_f64().unwrap(), 10000.5);
        assert_eq!(exec.price_f64().unwrap(), 155.25);
        assert_eq!(exec.loss_gain_f64().unwrap(), 1234.5);
        assert_eq!(exec.fee_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_deserialize_execution() {
        let json = r#"
        {
            "executionId": 72123911,
            "orderId": 123456789,
            "symbol": "USD_JPY",
            "side": "BUY",
            "settleType": "OPEN",
            "size": "10000",
            "price": "135.5",
            "lossGain": "0",
            "fee": "0",
            "timestamp": "2019-03-19T02:15:06.064Z"
        }
        "#;
        let exec: Execution = serde_json::from_str(json).unwrap();
        assert_eq!(exec.execution_id, 72123911);
        assert_eq!(exec.symbol, "USD_JPY");
    }
}
