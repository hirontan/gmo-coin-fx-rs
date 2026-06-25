use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub size: String,

    #[serde(rename = "clientOrderId", skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    #[serde(rename = "executionType")]
    pub execution_type: ExecutionType,

    #[serde(rename = "limitPrice", skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,

    #[serde(rename = "stopPrice", skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,

    #[serde(rename = "lowerBound", skip_serializing_if = "Option::is_none")]
    pub lower_bound: Option<String>,

    #[serde(rename = "upperBound", skip_serializing_if = "Option::is_none")]
    pub upper_bound: Option<String>,
}

impl OrderRequest {
    pub fn builder() -> OrderRequestBuilder {
        OrderRequestBuilder::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct OrderRequestBuilder {
    symbol: Option<String>,
    side: Option<OrderSide>,
    size: Option<String>,
    client_order_id: Option<String>,
    execution_type: Option<ExecutionType>,
    limit_price: Option<String>,
    stop_price: Option<String>,
    lower_bound: Option<String>,
    upper_bound: Option<String>,
}

impl OrderRequestBuilder {
    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    pub fn size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn client_order_id(mut self, client_order_id: impl Into<String>) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }

    pub fn execution_type(mut self, execution_type: ExecutionType) -> Self {
        self.execution_type = Some(execution_type);
        self
    }

    pub fn limit_price(mut self, limit_price: impl Into<String>) -> Self {
        self.limit_price = Some(limit_price.into());
        self
    }

    pub fn stop_price(mut self, stop_price: impl Into<String>) -> Self {
        self.stop_price = Some(stop_price.into());
        self
    }

    pub fn lower_bound(mut self, lower_bound: impl Into<String>) -> Self {
        self.lower_bound = Some(lower_bound.into());
        self
    }

    pub fn upper_bound(mut self, upper_bound: impl Into<String>) -> Self {
        self.upper_bound = Some(upper_bound.into());
        self
    }

    pub fn build(self) -> Result<OrderRequest, crate::error::GmoFxError> {
        let symbol = self.symbol.ok_or_else(|| {
            crate::error::GmoFxError::InvalidRequest("symbol is required".to_string())
        })?;
        let side = self.side.ok_or_else(|| {
            crate::error::GmoFxError::InvalidRequest("side is required".to_string())
        })?;
        let size = self.size.ok_or_else(|| {
            crate::error::GmoFxError::InvalidRequest("size is required".to_string())
        })?;
        let execution_type = self.execution_type.ok_or_else(|| {
            crate::error::GmoFxError::InvalidRequest("execution_type is required".to_string())
        })?;

        Ok(OrderRequest {
            symbol,
            side,
            size,
            client_order_id: self.client_order_id,
            execution_type,
            limit_price: self.limit_price,
            stop_price: self.stop_price,
            lower_bound: self.lower_bound,
            upper_bound: self.upper_bound,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CancelOrderRequest {
    #[serde(rename = "orderId")]
    pub order_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangeOrderRequest {
    #[serde(rename = "orderId")]
    pub order_id: u64,
    pub price: String,
    #[serde(rename = "losscutPrice", skip_serializing_if = "Option::is_none")]
    pub losscut_price: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CancelBulkOrderRequest {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[serde(rename = "settleType", skip_serializing_if = "Option::is_none")]
    pub settle_type: Option<SettleType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CloseOrderRequest {
    #[serde(rename = "positionId")]
    pub position_id: u64,
    pub size: String,
    #[serde(rename = "executionType")]
    pub execution_type: ExecutionType,
    #[serde(rename = "timeInForce", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "cancelBefore", skip_serializing_if = "Option::is_none")]
    pub cancel_before: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CloseBulkOrderRequest {
    pub symbol: String,
    pub side: String,
    #[serde(rename = "executionType")]
    pub execution_type: ExecutionType,
    #[serde(rename = "timeInForce", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpeedOrderRequest {
    pub symbol: String,
    pub side: String,
    pub size: String,
    #[serde(rename = "sizeStep", skip_serializing_if = "Option::is_none")]
    pub size_step: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    BUY,
    SELL,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionType {
    MARKET,
    LIMIT,
    STOP,
    OCO,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettleType {
    Open,
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "rootOrderId")]
    pub root_order_id: u64,

    #[serde(rename = "clientOrderId")]
    pub client_order_id: Option<String>,

    #[serde(rename = "orderId")]
    pub order_id: u64,

    pub symbol: String,
    pub side: String,

    #[serde(rename = "orderType")]
    pub order_type: String,

    #[serde(rename = "executionType")]
    pub execution_type: String,

    #[serde(rename = "settleType")]
    pub settle_type: SettleType,

    pub size: String,

    #[serde(default)]
    pub price: Option<String>,

    pub status: String,

    #[serde(default)]
    pub expiry: Option<String>,

    pub timestamp: String,
}

impl Order {
    pub fn size_f64(&self) -> crate::Result<f64> {
        self.size.parse::<f64>().map_err(Into::into)
    }

    pub fn price_f64(&self) -> crate::Result<Option<f64>> {
        self.price
            .as_deref()
            .map(|p| p.parse::<f64>().map_err(Into::into))
            .transpose()
    }

    #[cfg(feature = "chrono")]
    pub fn parsed_timestamp(&self) -> crate::Result<chrono::DateTime<chrono::Utc>> {
        super::common::parse_timestamp(&self.timestamp)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveOrders {
    pub list: Vec<Order>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_request_builder_success() {
        let req = OrderRequest::builder()
            .symbol("USD_JPY")
            .side(OrderSide::BUY)
            .size("10000")
            .execution_type(ExecutionType::LIMIT)
            .limit_price("157.500")
            .client_order_id("client_123")
            .stop_price("156.000")
            .lower_bound("155.000")
            .upper_bound("160.000")
            .build()
            .unwrap();

        assert_eq!(req.symbol, "USD_JPY");
        assert!(matches!(req.side, OrderSide::BUY));
        assert_eq!(req.size, "10000");
        assert!(matches!(req.execution_type, ExecutionType::LIMIT));
        assert_eq!(req.limit_price, Some("157.500".to_string()));
        assert_eq!(req.client_order_id, Some("client_123".to_string()));
        assert_eq!(req.stop_price, Some("156.000".to_string()));
        assert_eq!(req.lower_bound, Some("155.000".to_string()));
        assert_eq!(req.upper_bound, Some("160.000".to_string()));
    }

    #[test]
    fn test_order_request_builder_missing_required() {
        let result = OrderRequest::builder()
            .side(OrderSide::BUY)
            .size("10000")
            .execution_type(ExecutionType::LIMIT)
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid request: symbol is required"
        );

        let result = OrderRequest::builder()
            .symbol("USD_JPY")
            .size("10000")
            .execution_type(ExecutionType::LIMIT)
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid request: side is required"
        );

        let result = OrderRequest::builder()
            .symbol("USD_JPY")
            .side(OrderSide::BUY)
            .execution_type(ExecutionType::LIMIT)
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid request: size is required"
        );

        let result = OrderRequest::builder()
            .symbol("USD_JPY")
            .side(OrderSide::BUY)
            .size("10000")
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid request: execution_type is required"
        );
    }

    #[test]
    fn test_change_order_request_serialization() {
        let req = ChangeOrderRequest {
            order_id: 12345,
            price: "150.50".to_string(),
            losscut_price: Some("148.00".to_string()),
        };
        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains(r#""orderId":12345"#));
        assert!(serialized.contains(r#""price":"150.50""#));
        assert!(serialized.contains(r#""losscutPrice":"148.00""#));

        let req_no_losscut = ChangeOrderRequest {
            order_id: 12345,
            price: "150.50".to_string(),
            losscut_price: None,
        };
        let serialized_no_losscut = serde_json::to_string(&req_no_losscut).unwrap();
        assert!(serialized_no_losscut.contains(r#""orderId":12345"#));
        assert!(serialized_no_losscut.contains(r#""price":"150.50""#));
        assert!(!serialized_no_losscut.contains("losscutPrice"));
    }

    #[test]
    fn test_order_f64() {
        let order = Order {
            root_order_id: 1,
            client_order_id: None,
            order_id: 2,
            symbol: "USD_JPY".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            execution_type: "LIMIT".to_string(),
            settle_type: SettleType::Open,
            size: "10000".to_string(),
            price: Some("155.25".to_string()),
            status: "ORDERED".to_string(),
            expiry: None,
            timestamp: "now".to_string(),
        };

        assert_eq!(order.size_f64().unwrap(), 10000.0);
        assert_eq!(order.price_f64().unwrap(), Some(155.25));

        let order_no_price = Order {
            price: None,
            ..order
        };
        assert_eq!(order_no_price.price_f64().unwrap(), None);
    }
}
