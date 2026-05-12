use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PublicWsMessage {
    Ticker(TickerEvent),
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickerEvent {
    pub symbol: String,
    pub ask: String,
    pub bid: String,
    pub timestamp: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "channel")]
pub enum PrivateWsMessage {
    #[serde(rename = "executionEvents")]
    Execution(ExecutionEvent),
    #[serde(rename = "positionEvents")]
    Position(PositionEvent),
    #[serde(rename = "orderEvents")]
    Order(OrderEvent),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionEvent {
    pub amount: String,
    #[serde(rename = "rootOrderId")]
    pub root_order_id: u64,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "clientOrderId", skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(rename = "executionId")]
    pub execution_id: u64,
    pub symbol: String,
    #[serde(rename = "settleType")]
    pub settle_type: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    #[serde(rename = "executionType")]
    pub execution_type: String,
    pub side: String,
    #[serde(rename = "executionPrice")]
    pub execution_price: String,
    #[serde(rename = "executionSize")]
    pub execution_size: String,
    #[serde(rename = "positionId")]
    pub position_id: u64,
    #[serde(rename = "lossGain")]
    pub loss_gain: String,
    #[serde(rename = "settledSwap", skip_serializing_if = "Option::is_none")]
    pub settled_swap: Option<String>,
    pub fee: String,
    #[serde(rename = "orderPrice")]
    pub order_price: String,
    #[serde(rename = "orderExecutedSize")]
    pub order_executed_size: String,
    #[serde(rename = "orderSize")]
    pub order_size: String,
    #[serde(rename = "msgType")]
    pub msg_type: String,
    #[serde(rename = "orderTimestamp")]
    pub order_timestamp: String,
    #[serde(rename = "executionTimestamp")]
    pub execution_timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PositionEvent {
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
    pub timestamp: String,
    #[serde(rename = "totalSwap", skip_serializing_if = "Option::is_none")]
    pub total_swap: Option<String>,
    #[serde(rename = "msgType")]
    pub msg_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderEvent {
    #[serde(rename = "rootOrderId")]
    pub root_order_id: u64,
    #[serde(rename = "clientOrderId", skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    pub symbol: String,
    #[serde(rename = "settleType")]
    pub settle_type: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    #[serde(rename = "executionType")]
    pub execution_type: String,
    pub side: String,
    #[serde(rename = "orderStatus")]
    pub order_status: String,
    #[serde(rename = "orderTimestamp")]
    pub order_timestamp: String,
    #[serde(rename = "orderPrice")]
    pub order_price: String,
    #[serde(rename = "orderSize")]
    pub order_size: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(rename = "msgType")]
    pub msg_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_public_message() {
        let json = r#"{"symbol":"USD_JPY","ask":"157.266","bid":"157.261","timestamp":"2026-05-01T06:06:33.584446Z","status":"OPEN"}"#;
        let msg: PublicWsMessage = serde_json::from_str(json).unwrap();
        match msg {
            PublicWsMessage::Ticker(t) => {
                assert_eq!(t.symbol, "USD_JPY");
                assert_eq!(t.ask, "157.266");
            }
        }
    }

    #[test]
    fn test_deserialize_private_message_execution() {
        let json = r#"
        {
          "channel": "executionEvents",
          "amount": "-30",
          "rootOrderId": 123456789,
          "orderId": 123456789,
          "clientOrderId": "abc123",
          "executionId": 72123911,
          "symbol": "USD_JPY",
          "settleType": "OPEN",
          "orderType": "NORMAL",
          "executionType": "LIMIT",
          "side": "BUY",
          "executionPrice": "138.963",
          "executionSize": "10000",
          "positionId": 123456789,
          "lossGain": "0",
          "settledSwap": "0",
          "fee": "-30",
          "orderPrice": "140",
          "orderExecutedSize": "10000",
          "orderSize": "10000",
          "msgType": "ER",
          "orderTimestamp": "2019-03-19T02:15:06.081Z",
          "executionTimestamp": "2019-03-19T02:15:06.081Z"
        }
        "#;
        let msg: PrivateWsMessage = serde_json::from_str(json).unwrap();
        match msg {
            PrivateWsMessage::Execution(e) => {
                assert_eq!(e.execution_id, 72123911);
                assert_eq!(e.symbol, "USD_JPY");
            }
            _ => panic!("Expected execution event"),
        }
    }
}
