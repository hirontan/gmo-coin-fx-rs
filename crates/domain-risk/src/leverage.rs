use crate::error::{Result, RiskError};
use crate::position_size::notional_value;

/// 実効レバレッジの計算を行います。
pub struct LeverageCalculator;

/// Calculate the effective leverage directly from quantity, price, and equity.
pub fn effective_leverage(quantity: f64, price: f64, equity: f64) -> Result<f64> {
    if equity <= 0.0 {
        return Err(RiskError::InvalidEquity(equity));
    }
    let notional = notional_value(quantity, price)?;
    Ok(notional / equity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_leverage_valid() {
        let el = effective_leverage(20_000.0, 157.56, 300_000.0).unwrap();
        assert_eq!(el, 10.504);
    }

    #[test]
    fn test_effective_leverage_invalid() {
        assert_eq!(
            effective_leverage(20_000.0, 157.56, 0.0),
            Err(RiskError::InvalidEquity(0.0))
        );
        assert_eq!(
            effective_leverage(0.0, 157.56, 300_000.0),
            Err(RiskError::InvalidQuantity(0.0))
        );
    }
}
