use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: i64,
    pub data: T,

    #[serde(default)]
    pub messages: Option<Vec<crate::ApiMessage>>,

    #[serde(default)]
    pub responsetime: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccountAsset, Ticker};

    #[test]
    fn test_equality_comparison() {
        let ticker1 = Ticker {
            symbol: "USD_JPY".to_string(),
            ask: "150.00".to_string(),
            bid: "149.95".to_string(),
            timestamp: "2026-06-22T00:00:00Z".to_string(),
            status: "OPEN".to_string(),
        };
        let ticker2 = ticker1.clone();
        assert_eq!(ticker1, ticker2);

        let asset1 = AccountAsset {
            equity: "1000000".to_string(),
            available_amount: "900000".to_string(),
            balance: "1000000".to_string(),
            estimated_trade_fee: "0".to_string(),
            margin: "100000".to_string(),
            margin_ratio: "1000".to_string(),
            position_loss_gain: "0".to_string(),
            total_swap: "0".to_string(),
            transferable_amount: "900000".to_string(),
        };
        let asset2 = asset1.clone();
        assert_eq!(asset1, asset2);

        let response1 = ApiResponse {
            status: 0,
            data: ticker1.clone(),
            messages: None,
            responsetime: Some("2026-06-22T00:00:00Z".to_string()),
        };
        let response2 = response1.clone();
        assert_eq!(response1, response2);
    }
}
