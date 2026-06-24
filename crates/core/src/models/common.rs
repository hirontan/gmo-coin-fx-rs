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

#[cfg(feature = "chrono")]
pub fn parse_timestamp(s: &str) -> crate::Result<chrono::DateTime<chrono::Utc>> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&chrono::Utc));
    }
    if let Ok(ms) = s.parse::<i64>() {
        if let Some(dt) =
            chrono::DateTime::from_timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32)
        {
            return Ok(dt);
        }
    }
    Err(crate::error::GmoFxError::InvalidRequest(format!(
        "failed to parse timestamp: {}",
        s
    )))
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

    #[test]
    #[cfg(feature = "chrono")]
    fn test_timestamp_parsing() {
        let rfc3339_str = "2026-05-01T06:06:33.584446Z";
        let parsed1 = parse_timestamp(rfc3339_str).unwrap();
        assert_eq!(parsed1.to_rfc3339(), "2026-05-01T06:06:33.584446+00:00");

        let millis_str = "1588303593584";
        let parsed2 = parse_timestamp(millis_str).unwrap();
        assert_eq!(parsed2.to_rfc3339(), "2020-05-01T03:26:33.584+00:00");

        assert!(parse_timestamp("invalid").is_err());
        assert!(parse_timestamp("10:00").is_err());

        let ticker = Ticker {
            symbol: "USD_JPY".to_string(),
            ask: "150.00".to_string(),
            bid: "149.95".to_string(),
            timestamp: rfc3339_str.to_string(),
            status: "OPEN".to_string(),
        };
        assert_eq!(ticker.parsed_timestamp().unwrap(), parsed1);
    }
}
