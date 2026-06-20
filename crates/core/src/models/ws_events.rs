use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PublicWsMessage {
    Ticker(TickerEvent),
    OrderBook(OrderBookEvent),
}

impl PublicWsMessage {
    pub fn channel(&self) -> &str {
        match self {
            PublicWsMessage::Ticker(_) => "ticker",
            PublicWsMessage::OrderBook(_) => "orderbooks",
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            PublicWsMessage::Ticker(t) => &t.symbol,
            PublicWsMessage::OrderBook(ob) => &ob.symbol,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickerEvent {
    pub symbol: String,
    pub ask: String,
    pub bid: String,
    pub timestamp: String,
    pub status: String,
}

impl TickerEvent {
    pub fn ask_f64(&self) -> crate::Result<f64> {
        self.ask.parse::<f64>().map_err(Into::into)
    }

    pub fn bid_f64(&self) -> crate::Result<f64> {
        self.bid.parse::<f64>().map_err(Into::into)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderBookEntry {
    pub price: String,
    pub size: String,
}

impl OrderBookEntry {
    pub fn price_f64(&self) -> crate::Result<f64> {
        self.price.parse::<f64>().map_err(Into::into)
    }

    pub fn size_f64(&self) -> crate::Result<f64> {
        self.size.parse::<f64>().map_err(Into::into)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderBookEvent {
    pub symbol: String,
    pub asks: Vec<OrderBookEntry>,
    pub bids: Vec<OrderBookEntry>,
    pub timestamp: String,
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

impl PrivateWsMessage {
    pub fn channel(&self) -> &str {
        match self {
            PrivateWsMessage::Execution(_) => "executionEvents",
            PrivateWsMessage::Position(_) => "positionEvents",
            PrivateWsMessage::Order(_) => "orderEvents",
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            PrivateWsMessage::Execution(e) => &e.symbol,
            PrivateWsMessage::Position(p) => &p.symbol,
            PrivateWsMessage::Order(o) => &o.symbol,
        }
    }
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

impl ExecutionEvent {
    pub fn amount_f64(&self) -> crate::Result<f64> {
        self.amount.parse::<f64>().map_err(Into::into)
    }

    pub fn execution_price_f64(&self) -> crate::Result<f64> {
        self.execution_price.parse::<f64>().map_err(Into::into)
    }

    pub fn execution_size_f64(&self) -> crate::Result<f64> {
        self.execution_size.parse::<f64>().map_err(Into::into)
    }

    pub fn loss_gain_f64(&self) -> crate::Result<f64> {
        self.loss_gain.parse::<f64>().map_err(Into::into)
    }

    pub fn settled_swap_f64(&self) -> crate::Result<Option<f64>> {
        self.settled_swap
            .as_deref()
            .map(|p| p.parse::<f64>().map_err(Into::into))
            .transpose()
    }

    pub fn fee_f64(&self) -> crate::Result<f64> {
        self.fee.parse::<f64>().map_err(Into::into)
    }

    pub fn order_price_f64(&self) -> crate::Result<f64> {
        self.order_price.parse::<f64>().map_err(Into::into)
    }

    pub fn order_executed_size_f64(&self) -> crate::Result<f64> {
        self.order_executed_size.parse::<f64>().map_err(Into::into)
    }

    pub fn order_size_f64(&self) -> crate::Result<f64> {
        self.order_size.parse::<f64>().map_err(Into::into)
    }
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

impl PositionEvent {
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

    pub fn total_swap_f64(&self) -> crate::Result<Option<f64>> {
        self.total_swap
            .as_deref()
            .map(|p| p.parse::<f64>().map_err(Into::into))
            .transpose()
    }
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

impl OrderEvent {
    pub fn order_price_f64(&self) -> crate::Result<f64> {
        self.order_price.parse::<f64>().map_err(Into::into)
    }

    pub fn order_size_f64(&self) -> crate::Result<f64> {
        self.order_size.parse::<f64>().map_err(Into::into)
    }
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
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn test_deserialize_public_message_orderbook() {
        let json = r#"{"symbol":"BTC_JPY","asks":[{"price":"10000000","size":"0.5"}],"bids":[{"price":"9999000","size":"1.2"}],"timestamp":"2026-05-01T06:06:33.584446Z"}"#;
        let msg: PublicWsMessage = serde_json::from_str(json).unwrap();
        match msg {
            PublicWsMessage::OrderBook(ob) => {
                assert_eq!(ob.symbol, "BTC_JPY");
                assert_eq!(ob.asks[0].price, "10000000");
                assert_eq!(ob.asks[0].price_f64().unwrap(), 10000000.0);
                assert_eq!(ob.asks[0].size_f64().unwrap(), 0.5);
                assert_eq!(ob.bids[0].price, "9999000");
                assert_eq!(ob.bids[0].price_f64().unwrap(), 9999000.0);
                assert_eq!(ob.bids[0].size_f64().unwrap(), 1.2);
            }
            _ => panic!("Expected OrderBook event"),
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

    #[test]
    fn test_ws_events_f64() {
        let ticker = TickerEvent {
            symbol: "USD_JPY".to_string(),
            ask: "155.25".to_string(),
            bid: "155.20".to_string(),
            timestamp: "now".to_string(),
            status: "OPEN".to_string(),
        };
        assert_eq!(ticker.ask_f64().unwrap(), 155.25);
        assert_eq!(ticker.bid_f64().unwrap(), 155.20);

        let exec = ExecutionEvent {
            amount: "-30".to_string(),
            root_order_id: 1,
            order_id: 2,
            client_order_id: None,
            execution_id: 3,
            symbol: "USD_JPY".to_string(),
            settle_type: "OPEN".to_string(),
            order_type: "NORMAL".to_string(),
            execution_type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            execution_price: "138.963".to_string(),
            execution_size: "10000".to_string(),
            position_id: 4,
            loss_gain: "100".to_string(),
            settled_swap: Some("10".to_string()),
            fee: "-30".to_string(),
            order_price: "140".to_string(),
            order_executed_size: "10000".to_string(),
            order_size: "10000".to_string(),
            msg_type: "ER".to_string(),
            order_timestamp: "now".to_string(),
            execution_timestamp: "now".to_string(),
        };
        assert_eq!(exec.amount_f64().unwrap(), -30.0);
        assert_eq!(exec.execution_price_f64().unwrap(), 138.963);
        assert_eq!(exec.execution_size_f64().unwrap(), 10000.0);
        assert_eq!(exec.loss_gain_f64().unwrap(), 100.0);
        assert_eq!(exec.settled_swap_f64().unwrap(), Some(10.0));
        assert_eq!(exec.fee_f64().unwrap(), -30.0);
        assert_eq!(exec.order_price_f64().unwrap(), 140.0);
        assert_eq!(exec.order_executed_size_f64().unwrap(), 10000.0);
        assert_eq!(exec.order_size_f64().unwrap(), 10000.0);

        let pos = PositionEvent {
            position_id: 1,
            symbol: "USD_JPY".to_string(),
            side: "BUY".to_string(),
            size: "10000".to_string(),
            ordered_size: "0".to_string(),
            price: "135.5".to_string(),
            loss_gain: "1000".to_string(),
            timestamp: "now".to_string(),
            total_swap: Some("50".to_string()),
            msg_type: "ER".to_string(),
        };
        assert_eq!(pos.size_f64().unwrap(), 10000.0);
        assert_eq!(pos.ordered_size_f64().unwrap(), 0.0);
        assert_eq!(pos.price_f64().unwrap(), 135.5);
        assert_eq!(pos.loss_gain_f64().unwrap(), 1000.0);
        assert_eq!(pos.total_swap_f64().unwrap(), Some(50.0));

        let order = OrderEvent {
            root_order_id: 1,
            client_order_id: None,
            order_id: 2,
            symbol: "USD_JPY".to_string(),
            settle_type: "OPEN".to_string(),
            order_type: "NORMAL".to_string(),
            execution_type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            order_status: "ORDERED".to_string(),
            order_timestamp: "now".to_string(),
            order_price: "140".to_string(),
            order_size: "10000".to_string(),
            expiry: None,
            msg_type: "ER".to_string(),
        };
        assert_eq!(order.order_price_f64().unwrap(), 140.0);
        assert_eq!(order.order_size_f64().unwrap(), 10000.0);
    }
}
