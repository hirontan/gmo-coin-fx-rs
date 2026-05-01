use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct PositionsList {
    pub list: Vec<Position>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct PositionSummaryList {
    pub list: Vec<PositionSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
