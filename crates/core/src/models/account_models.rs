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

impl AccountAsset {
    pub fn equity_f64(&self) -> crate::Result<f64> {
        self.equity.parse::<f64>().map_err(Into::into)
    }

    pub fn available_amount_f64(&self) -> crate::Result<f64> {
        self.available_amount.parse::<f64>().map_err(Into::into)
    }

    pub fn balance_f64(&self) -> crate::Result<f64> {
        self.balance.parse::<f64>().map_err(Into::into)
    }

    pub fn estimated_trade_fee_f64(&self) -> crate::Result<f64> {
        self.estimated_trade_fee.parse::<f64>().map_err(Into::into)
    }

    pub fn margin_f64(&self) -> crate::Result<f64> {
        self.margin.parse::<f64>().map_err(Into::into)
    }

    pub fn margin_ratio_f64(&self) -> crate::Result<f64> {
        self.margin_ratio.parse::<f64>().map_err(Into::into)
    }

    pub fn position_loss_gain_f64(&self) -> crate::Result<f64> {
        self.position_loss_gain.parse::<f64>().map_err(Into::into)
    }

    pub fn total_swap_f64(&self) -> crate::Result<f64> {
        self.total_swap.parse::<f64>().map_err(Into::into)
    }

    pub fn transferable_amount_f64(&self) -> crate::Result<f64> {
        self.transferable_amount.parse::<f64>().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_asset_f64() {
        let asset = AccountAsset {
            equity: "1000000.50".to_string(),
            available_amount: "950000.00".to_string(),
            balance: "1000000.00".to_string(),
            estimated_trade_fee: "150.00".to_string(),
            margin: "50000.00".to_string(),
            margin_ratio: "20.00".to_string(),
            position_loss_gain: "-500.00".to_string(),
            total_swap: "120.00".to_string(),
            transferable_amount: "900000.00".to_string(),
        };

        assert_eq!(asset.equity_f64().unwrap(), 1000000.50);
        assert_eq!(asset.available_amount_f64().unwrap(), 950000.00);
        assert_eq!(asset.balance_f64().unwrap(), 1000000.00);
        assert_eq!(asset.estimated_trade_fee_f64().unwrap(), 150.00);
        assert_eq!(asset.margin_f64().unwrap(), 50000.00);
        assert_eq!(asset.margin_ratio_f64().unwrap(), 20.00);
        assert_eq!(asset.position_loss_gain_f64().unwrap(), -500.00);
        assert_eq!(asset.total_swap_f64().unwrap(), 120.00);
        assert_eq!(asset.transferable_amount_f64().unwrap(), 900000.00);
    }
}

