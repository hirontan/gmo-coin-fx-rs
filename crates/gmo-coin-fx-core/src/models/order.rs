use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    BUY,
    SELL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionType {
    MARKET,
    LIMIT,
    STOP,
    OCO,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub settle_type: String,

    pub size: String,

    #[serde(default)]
    pub price: Option<String>,

    pub status: String,

    #[serde(default)]
    pub expiry: Option<String>,

    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActiveOrders {
    pub list: Vec<Order>,
}
