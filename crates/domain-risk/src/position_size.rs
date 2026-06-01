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

/// Calculate the stop-loss price distance based on equity, risk percentage, and quantity.
///
/// This is the inverse of `max_quantity_by_risk`.
pub fn stop_distance_from_risk(equity: f64, risk_per_trade_pct: f64, quantity: f64) -> Result<f64> {
    if quantity <= 0.0 {
        return Err(RiskError::InvalidQuantity(quantity));
    }
    let amount = risk_amount(equity, risk_per_trade_pct)?;
    Ok(amount / quantity)
}

/// Calculate the take-profit price distance based on stop distance and risk-reward ratio.
pub fn take_profit_distance(stop_distance: f64, risk_reward_ratio: f64) -> Result<f64> {
    if stop_distance <= 0.0 {
        return Err(RiskError::InvalidStopDistance(stop_distance));
    }
    if risk_reward_ratio <= 0.0 {
        return Err(RiskError::InvalidRiskRewardRatio(risk_reward_ratio));
    }
    Ok(stop_distance * risk_reward_ratio)
}

/// Get the pip size for a symbol (e.g., 0.01 for USD/JPY, 0.0001 for EUR/USD).
pub fn pip_size(symbol: &str) -> f64 {
    let symbol_upper = symbol.to_uppercase();
    if symbol_upper.ends_with("JPY") || symbol_upper.ends_with("_JPY") {
        0.01
    } else {
        0.0001
    }
}

/// Calculate the pip value in quote currency for a given quantity and pip size.
pub fn pip_value(quantity: f64, pip_size: f64) -> f64 {
    quantity.abs() * pip_size
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

/// Round down the quantity to the nearest multiple of the trading unit.
pub fn round_down_to_unit(quantity: f64, unit: f64) -> Result<f64> {
    if unit <= 0.0 {
        return Err(RiskError::InvalidUnit(unit));
    }
    if quantity < 0.0 {
        return Err(RiskError::InvalidQuantity(quantity));
    }
    Ok((quantity / unit).floor() * unit)
}

/// Calculate trailing stop distance from ATR.
pub fn trailing_stop_from_atr(atr: f64, multiplier: f64) -> Result<f64> {
    if atr <= 0.0 {
        return Err(RiskError::InvalidAtr(atr));
    }
    if multiplier <= 0.0 {
        return Err(RiskError::InvalidMultiplier(multiplier));
    }
    Ok(atr * multiplier)
}

/// Calculate trailing stop distance from fixed percentage of current price.
pub fn trailing_stop_from_pct(price: f64, pct: f64) -> Result<f64> {
    if price <= 0.0 {
        return Err(RiskError::InvalidPrice(price));
    }
    if pct <= 0.0 || pct > 1.0 {
        return Err(RiskError::InvalidRiskPct(pct));
    }
    Ok(price * pct)
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
        assert!(
            (qty - 5712.109).abs() < 0.01,
            "Expected ~5712.1, got {}",
            qty
        );
    }

    #[test]
    fn test_max_quantity_by_leverage_invalid() {
        assert_eq!(
            max_quantity_by_leverage(0.0, 3.0, 157.56),
            Err(RiskError::InvalidEquity(0.0))
        );
        assert_eq!(
            max_quantity_by_leverage(300_000.0, 0.0, 157.56),
            Err(RiskError::InvalidLeverage(0.0))
        );
        assert_eq!(
            max_quantity_by_leverage(300_000.0, 3.0, -1.0),
            Err(RiskError::InvalidPrice(-1.0))
        );
    }

    #[test]
    fn test_round_down_to_unit_valid() {
        assert_eq!(round_down_to_unit(5711.0, 1000.0).unwrap(), 5000.0);
        assert_eq!(round_down_to_unit(1666.0, 1000.0).unwrap(), 1000.0);
        assert_eq!(round_down_to_unit(999.0, 1000.0).unwrap(), 0.0);
    }

    #[test]
    fn test_round_down_to_unit_invalid() {
        assert_eq!(
            round_down_to_unit(5711.0, 0.0),
            Err(RiskError::InvalidUnit(0.0))
        );
        assert_eq!(
            round_down_to_unit(-100.0, 1000.0),
            Err(RiskError::InvalidQuantity(-100.0))
        );
    }

    #[test]
    fn test_stop_distance_from_risk_valid() {
        let dist = stop_distance_from_risk(300_000.0, 0.005, 10_000.0).unwrap();
        assert_eq!(dist, 0.15);
    }

    #[test]
    fn test_stop_distance_from_risk_invalid() {
        assert_eq!(
            stop_distance_from_risk(300_000.0, 0.005, 0.0),
            Err(RiskError::InvalidQuantity(0.0))
        );
        assert_eq!(
            stop_distance_from_risk(300_000.0, 0.005, -100.0),
            Err(RiskError::InvalidQuantity(-100.0))
        );
        assert_eq!(
            stop_distance_from_risk(0.0, 0.005, 10_000.0),
            Err(RiskError::InvalidEquity(0.0))
        );
        assert_eq!(
            stop_distance_from_risk(300_000.0, 1.5, 10_000.0),
            Err(RiskError::InvalidRiskPct(1.5))
        );
    }

    #[test]
    fn test_take_profit_distance_valid() {
        let dist = take_profit_distance(0.50, 2.0).unwrap();
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_take_profit_distance_invalid() {
        assert_eq!(
            take_profit_distance(0.0, 2.0),
            Err(RiskError::InvalidStopDistance(0.0))
        );
        assert_eq!(
            take_profit_distance(-0.5, 2.0),
            Err(RiskError::InvalidStopDistance(-0.5))
        );
        assert_eq!(
            take_profit_distance(0.5, 0.0),
            Err(RiskError::InvalidRiskRewardRatio(0.0))
        );
        assert_eq!(
            take_profit_distance(0.5, -1.0),
            Err(RiskError::InvalidRiskRewardRatio(-1.0))
        );
    }

    #[test]
    fn test_pip_size_cross_yen() {
        assert_eq!(pip_size("USD_JPY"), 0.01);
        assert_eq!(pip_size("EUR_JPY"), 0.01);
        assert_eq!(pip_size("GBP_JPY"), 0.01);
        assert_eq!(pip_size("USDJPY"), 0.01);
        assert_eq!(pip_size("eur_jpy"), 0.01);
    }

    #[test]
    fn test_pip_size_non_yen() {
        assert_eq!(pip_size("EUR_USD"), 0.0001);
        assert_eq!(pip_size("GBP_USD"), 0.0001);
        assert_eq!(pip_size("eur_usd"), 0.0001);
    }

    #[test]
    fn test_pip_value_long() {
        let val = pip_value(10_000.0, 0.01);
        assert_eq!(val, 100.0);
        let val_non_yen = pip_value(10_000.0, 0.0001);
        assert_eq!(val_non_yen, 1.0);
    }

    #[test]
    fn test_pip_value_short() {
        let val = pip_value(-10_000.0, 0.01);
        assert_eq!(val, 100.0);
        let val_non_yen = pip_value(-10_000.0, 0.0001);
        assert_eq!(val_non_yen, 1.0);
    }
}
