use crate::types::RiskMetrics;

/// 注文前のリスク判定（注文可否判定）を行います。
///
/// （プレースホルダー: 詳細な計算ロジックは対象外）
pub struct RiskGuard;

/// 建玉数量・価格・有効証拠金・口座レバレッジから、主要なリスク指標をまとめて計算します。
pub fn calculate_risk_metrics(
    equity: f64,
    quantity: f64,
    price: f64,
    leverage: f64,
) -> RiskMetrics {
    let notional = crate::position_size::notional_value(quantity, price).unwrap_or(0.0);
    let required = crate::margin::required_margin(quantity, price, leverage).unwrap_or(0.0);
    let eff_lev = crate::leverage::effective_leverage(quantity, price, equity).unwrap_or(0.0);
    let rate = crate::margin::margin_rate(equity, required).unwrap_or(0.0);

    RiskMetrics {
        notional_value: notional,
        required_margin: required,
        effective_leverage: eff_lev,
        margin_rate: rate,
        loss_per_1yen: quantity.abs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_risk_metrics_usd_jpy_example() {
        let equity = 300_000.0;
        let quantity = 20_000.0;
        let price = 157.56;
        let leverage = 25.0;

        let metrics = calculate_risk_metrics(equity, quantity, price, leverage);

        assert_eq!(metrics.notional_value, 3_151_200.0);
        assert_eq!(metrics.required_margin, 126_048.0);
        assert_eq!(metrics.effective_leverage, 10.504);
        // Assert that the margin rate is close to 237.99% (within 0.02% tolerance)
        assert!((metrics.margin_rate - 237.99).abs() < 0.02);
        assert_eq!(metrics.loss_per_1yen, 20_000.0);
    }

    #[test]
    fn test_calculate_risk_metrics_invalid_inputs() {
        // Zero equity case: effective leverage and margin rate should be 0 instead of panicking/division by zero
        let metrics = calculate_risk_metrics(0.0, 20_000.0, 157.56, 25.0);
        assert_eq!(metrics.notional_value, 3_151_200.0);
        assert_eq!(metrics.required_margin, 126_048.0);
        assert_eq!(metrics.effective_leverage, 0.0);
        assert_eq!(metrics.margin_rate, 0.0);
        assert_eq!(metrics.loss_per_1yen, 20_000.0);
    }
}
