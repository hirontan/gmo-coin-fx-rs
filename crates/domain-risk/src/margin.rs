use crate::error::{Result, RiskError};

/// 必要証拠金および証拠金維持率の計算を行います。
pub struct MarginCalculator;

/// Calculate the required margin for a position (quantity * price / leverage).
pub fn required_margin(quantity: f64, price: f64, leverage: f64) -> Result<f64> {
    if quantity <= 0.0 {
        return Err(RiskError::InvalidQuantity(quantity));
    }
    if price <= 0.0 {
        return Err(RiskError::InvalidPrice(price));
    }
    if leverage <= 0.0 {
        return Err(RiskError::InvalidLeverage(leverage));
    }
    Ok((quantity * price) / leverage)
}

/// Calculate the margin maintenance rate (equity / required_margin * 100).
pub fn margin_rate(equity: f64, required_margin: f64) -> Result<f64> {
    if equity <= 0.0 {
        return Err(RiskError::InvalidEquity(equity));
    }
    if required_margin <= 0.0 {
        return Err(RiskError::InvalidMargin(required_margin));
    }
    Ok((equity / required_margin) * 100.0)
}

/// Calculate the current drawdown percentage from peak equity and current equity.
pub fn drawdown_pct(peak_equity: f64, current_equity: f64) -> Result<f64> {
    if peak_equity <= 0.0 {
        return Err(RiskError::InvalidEquity(peak_equity));
    }
    if current_equity < 0.0 {
        return Err(RiskError::InvalidEquity(current_equity));
    }
    if current_equity >= peak_equity {
        return Ok(0.0);
    }
    Ok(((peak_equity - current_equity) / peak_equity) * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_margin_valid() {
        let margin = required_margin(20_000.0, 157.56, 25.0).unwrap();
        assert_eq!(margin, 126_048.0);
    }

    #[test]
    fn test_required_margin_invalid() {
        assert_eq!(
            required_margin(0.0, 157.56, 25.0),
            Err(RiskError::InvalidQuantity(0.0))
        );
        assert_eq!(
            required_margin(20_000.0, 0.0, 25.0),
            Err(RiskError::InvalidPrice(0.0))
        );
        assert_eq!(
            required_margin(20_000.0, 157.56, -25.0),
            Err(RiskError::InvalidLeverage(-25.0))
        );
    }

    #[test]
    fn test_margin_rate_valid() {
        let rate = margin_rate(300_000.0, 126_048.0).unwrap();
        assert!(
            (rate - 237.99).abs() < 0.02,
            "Expected ~237.99%, got {}",
            rate
        );
    }

    #[test]
    fn test_margin_rate_invalid() {
        assert_eq!(
            margin_rate(0.0, 126_048.0),
            Err(RiskError::InvalidEquity(0.0))
        );
        assert_eq!(
            margin_rate(300_000.0, 0.0),
            Err(RiskError::InvalidMargin(0.0))
        );
    }

    #[test]
    fn test_drawdown_pct_valid() {
        let draw = drawdown_pct(350_000.0, 300_000.0).unwrap();
        assert!((draw - 14.29).abs() < 0.01);
    }

    #[test]
    fn test_drawdown_pct_no_drawdown() {
        assert_eq!(drawdown_pct(300_000.0, 300_000.0).unwrap(), 0.0);
        assert_eq!(drawdown_pct(300_000.0, 350_000.0).unwrap(), 0.0);
    }

    #[test]
    fn test_drawdown_pct_invalid() {
        assert_eq!(
            drawdown_pct(0.0, 300_000.0),
            Err(RiskError::InvalidEquity(0.0))
        );
        assert_eq!(
            drawdown_pct(-100.0, 300_000.0),
            Err(RiskError::InvalidEquity(-100.0))
        );
        assert_eq!(
            drawdown_pct(300_000.0, -10.0),
            Err(RiskError::InvalidEquity(-10.0))
        );
    }
}
