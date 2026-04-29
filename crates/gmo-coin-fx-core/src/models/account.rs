use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AccountAsset {
    pub equity: String,

    #[serde(rename = "availableAmount")]
    pub available_amount: String,

    pub balance: String,

    #[serde(rename = "estimatedTradeFee")]
    pub estimated_trade_fee: String,

    pub margin: String,

    #[serde(rename = "marginRatio")]
    pub margin_ratio: String,

    #[serde(rename = "positionLossGain")]
    pub position_loss_gain: String,

    #[serde(rename = "totalSwap")]
    pub total_swap: String,

    #[serde(rename = "transferableAmount")]
    pub transferable_amount: String,
}
