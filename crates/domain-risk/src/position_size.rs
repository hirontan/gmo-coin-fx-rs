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

/// Calculate the risk amount based on equity and risk percentage.
pub fn risk_amount(equity: f64, risk_per_trade_pct: f64) -> Result<f64> {
    if equity <= 0.0 {
        return Err(RiskError::InvalidEquity(equity));
    }
    if risk_per_trade_pct <= 0.0 || risk_per_trade_pct > 1.0 {
        return Err(RiskError::InvalidRiskPct(risk_per_trade_pct));
    }
    Ok(equity * risk_per_trade_pct)
}

/// Calculate the maximum safe quantity based on risk amount and stop distance.
pub fn max_quantity_by_risk(
    equity: f64,
    risk_per_trade_pct: f64,
    stop_distance: f64,
) -> Result<f64> {
    if stop_distance <= 0.0 {
        return Err(RiskError::InvalidStopDistance(stop_distance));
    }
    let amount = risk_amount(equity, risk_per_trade_pct)?;
    Ok(amount / stop_distance)
}

/// Calculate the maximum quantity based on effective leverage limit.
pub fn max_quantity_by_leverage(
    equity: f64,
    max_effective_leverage: f64,
    price: f64,
) -> Result<f64> {
    if equity <= 0.0 {
        return Err(RiskError::InvalidEquity(equity));
    }
    if max_effective_leverage <= 0.0 {
        return Err(RiskError::InvalidLeverage(max_effective_leverage));
    }
    if price <= 0.0 {
        return Err(RiskError::InvalidPrice(price));
    }
    Ok((equity * max_effective_leverage) / price)
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

    #[test]
    fn test_risk_amount_valid() {
        let amt = risk_amount(300_000.0, 0.005).unwrap();
        assert_eq!(amt, 1_500.0);
    }

    #[test]
    fn test_max_quantity_by_risk_valid() {
        let qty = max_quantity_by_risk(300_000.0, 0.005, 0.50).unwrap();
        assert_eq!(qty, 3_000.0);
    }

    #[test]
    fn test_invalid_risk_amount() {
        assert_eq!(risk_amount(0.0, 0.005), Err(RiskError::InvalidEquity(0.0)));
        assert_eq!(
            risk_amount(300_000.0, 0.0),
            Err(RiskError::InvalidRiskPct(0.0))
        );
        assert_eq!(
            risk_amount(300_000.0, 1.5),
            Err(RiskError::InvalidRiskPct(1.5))
        );
    }

    #[test]
    fn test_invalid_max_quantity_by_risk() {
        assert_eq!(
            max_quantity_by_risk(300_000.0, 0.005, 0.0),
            Err(RiskError::InvalidStopDistance(0.0))
        );
    }

    #[test]
    fn test_max_quantity_by_leverage_valid() {
        let qty = max_quantity_by_leverage(300_000.0, 3.0, 157.56).unwrap();
        // Note: The issue stated 5711.0... but 300,000 * 3 / 157.56 is 5712.109...
        assert!((qty - 5712.109).abs() < 0.01, "Expected ~5712.1, got {}", qty);
    }

    #[test]
    fn test_max_quantity_by_leverage_invalid() {
        assert_eq!(max_quantity_by_leverage(0.0, 3.0, 157.56), Err(RiskError::InvalidEquity(0.0)));
        assert_eq!(max_quantity_by_leverage(300_000.0, 0.0, 157.56), Err(RiskError::InvalidLeverage(0.0)));
        assert_eq!(max_quantity_by_leverage(300_000.0, 3.0, -1.0), Err(RiskError::InvalidPrice(-1.0)));
    }
}
