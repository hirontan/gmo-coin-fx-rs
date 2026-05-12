use crate::error::{Result, RiskError};

/// 建玉数量のサイジングおよび評価総額の計算を行います。
pub struct PositionSizer;

/// Calculate the notional value of a position (quantity * price).
pub fn notional_value(quantity: f64, price: f64) -> Result<f64> {
    if quantity <= 0.0 {
        return Err(RiskError::InvalidQuantity(quantity));
    }
    if price <= 0.0 {
        return Err(RiskError::InvalidPrice(price));
    }
    Ok(quantity * price)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notional_value_valid() {
        let nv = notional_value(20_000.0, 157.56).unwrap();
        assert_eq!(nv, 3_151_200.0);
    }

    #[test]
    fn test_notional_value_invalid() {
        assert_eq!(
            notional_value(0.0, 157.56),
            Err(RiskError::InvalidQuantity(0.0))
        );
        assert_eq!(
            notional_value(20_000.0, -1.0),
            Err(RiskError::InvalidPrice(-1.0))
        );
    }
}
