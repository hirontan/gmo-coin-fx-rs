use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionsList {
    pub list: Vec<Execution>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
