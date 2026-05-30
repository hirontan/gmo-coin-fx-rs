use crate::types::{RiskCheckResult, RiskConfig, RiskMetrics};

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

/// Calculate total risk metrics across multiple open positions.
pub fn aggregate_risk_metrics(
    equity: f64,
    positions: &[(f64, f64, f64)],
    leverage: f64,
) -> RiskMetrics {
    let mut total_notional = 0.0;
    let mut total_required = 0.0;
    let mut total_loss_per_1yen = 0.0;

    for &(quantity, price, _unrealized_pnl) in positions {
        let abs_quantity = quantity.abs();
        let notional = crate::position_size::notional_value(abs_quantity, price).unwrap_or(0.0);
        let required = crate::margin::required_margin(abs_quantity, price, leverage).unwrap_or(0.0);
        total_notional += notional;
        total_required += required;
        total_loss_per_1yen += abs_quantity;
    }

    let effective_leverage = if equity <= 0.0 {
        0.0
    } else {
        total_notional / equity
    };

    let margin_rate = crate::margin::margin_rate(equity, total_required).unwrap_or(0.0);

    RiskMetrics {
        notional_value: total_notional,
        required_margin: total_required,
        effective_leverage,
        margin_rate,
        loss_per_1yen: total_loss_per_1yen,
    }
}

/// 注文前のリスク条件をチェックし、注文可能かどうかを判定します。
pub fn check_order_risk(
    equity: f64,
    quantity: f64,
    price: f64,
    account_leverage: f64,
    current_position_count: usize,
    config: RiskConfig,
) -> RiskCheckResult {
    let mut reasons = Vec::new();

    if quantity <= 0.0 {
        reasons.push("Quantity must be greater than 0".to_string());
    }
    if equity <= 0.0 {
        reasons.push("Equity must be greater than 0".to_string());
    }
    if price <= 0.0 {
        reasons.push("Price must be greater than 0".to_string());
    }
    if account_leverage <= 0.0 {
        reasons.push("Account leverage must be greater than 0".to_string());
    }

    let metrics = calculate_risk_metrics(equity, quantity, price, account_leverage);

    // Only perform limit checks if basic inputs are valid.
    if quantity > 0.0 && equity > 0.0 && price > 0.0 && account_leverage > 0.0 {
        if metrics.effective_leverage > config.max_effective_leverage {
            reasons.push(format!(
                "Effective leverage exceeds limit: {:.1}x > {:.1}x",
                metrics.effective_leverage, config.max_effective_leverage
            ));
        }
        if metrics.margin_rate < config.min_margin_rate {
            reasons.push(format!(
                "Margin maintenance rate is below threshold: {:.0}% < {:.0}%",
                metrics.margin_rate, config.min_margin_rate
            ));
        }
        if let Some(max_positions) = config.max_open_positions {
            if current_position_count >= max_positions {
                reasons.push(format!(
                    "Open position count exceeds limit: {} >= {}",
                    current_position_count, max_positions
                ));
            }
        }
    }

    let allowed = reasons.is_empty();

    RiskCheckResult {
        allowed,
        reasons,
        metrics,
    }
}

/// Check if the daily realized loss exceeds the maximum configured daily loss limit.
///
/// Returns `true` if the daily loss limit is exceeded (i.e., trading should be halted),
/// and `false` if the loss is within the limit (i.e., trading is allowed).
pub fn check_daily_loss_limit(daily_realized_pnl: f64, max_daily_loss: f64) -> bool {
    if max_daily_loss <= 0.0 {
        return false; // Limit is not configured or disabled
    }
    if daily_realized_pnl >= 0.0 {
        return false; // Daily PnL is in profit, limit not exceeded
    }
    daily_realized_pnl.abs() > max_daily_loss
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

    #[test]
    fn test_check_order_risk_rejected_example() {
        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 500.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        let result = check_order_risk(300_000.0, 20_000.0, 157.56, 25.0, 0, config);

        assert!(!result.allowed);
        assert_eq!(result.reasons.len(), 2);
        assert_eq!(
            result.reasons[0],
            "Effective leverage exceeds limit: 10.5x > 5.0x"
        );
        assert_eq!(
            result.reasons[1],
            "Margin maintenance rate is below threshold: 238% < 500%"
        );

        // Validate that metrics are included
        assert_eq!(result.metrics.notional_value, 3_151_200.0);
        assert_eq!(result.metrics.required_margin, 126_048.0);
        assert_eq!(result.metrics.effective_leverage, 10.504);
        assert!((result.metrics.margin_rate - 237.99).abs() < 0.02);
        assert_eq!(result.metrics.loss_per_1yen, 20_000.0);
    }

    #[test]
    fn test_check_order_risk_allowed() {
        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        // Smaller quantity to reduce effective leverage
        let result = check_order_risk(300_000.0, 5_000.0, 157.56, 25.0, 0, config);

        assert!(result.allowed);
        assert!(result.reasons.is_empty());
        assert_eq!(result.metrics.notional_value, 787_800.0);
        assert_eq!(result.metrics.required_margin, 31_512.0);
        assert_eq!(result.metrics.effective_leverage, 2.626);
        assert!((result.metrics.margin_rate - 952.01).abs() < 0.02);
    }

    #[test]
    fn test_check_order_risk_invalid_inputs() {
        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 500.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        let result = check_order_risk(300_000.0, 0.0, 157.56, 25.0, 0, config);

        assert!(!result.allowed);
        assert_eq!(result.reasons.len(), 1);
        assert_eq!(result.reasons[0], "Quantity must be greater than 0");
    }

    #[test]
    fn test_check_order_risk_position_limit_exceeded() {
        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: Some(3),
        };

        let result = check_order_risk(300_000.0, 5_000.0, 157.56, 25.0, 3, config);

        assert!(!result.allowed);
        assert_eq!(result.reasons.len(), 1);
        assert_eq!(
            result.reasons[0],
            "Open position count exceeds limit: 3 >= 3"
        );
    }

    #[test]
    fn test_check_order_risk_position_limit_not_exceeded() {
        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: Some(3),
        };

        let result = check_order_risk(300_000.0, 5_000.0, 157.56, 25.0, 2, config);

        assert!(result.allowed);
        assert!(result.reasons.is_empty());
    }

    #[test]
    fn test_aggregate_risk_metrics_multiple_positions() {
        let equity = 300_000.0;
        let positions = vec![
            (10_000.0, 150.0, 0.0), // Buy 10,000 USD/JPY at 150.0. Notional: 1,500,000. Required margin (at 25x): 60,000
            (-5_000.0, 150.0, 0.0), // Sell 5,000 USD/JPY at 150.0. Notional: 750,000. Required margin (at 25x): 30,000
        ];
        let leverage = 25.0;

        let metrics = aggregate_risk_metrics(equity, &positions, leverage);

        assert_eq!(metrics.notional_value, 2_250_000.0);
        assert_eq!(metrics.required_margin, 90_000.0);
        assert_eq!(metrics.effective_leverage, 7.5);
        assert!((metrics.margin_rate - 333.33).abs() < 0.02);
        assert_eq!(metrics.loss_per_1yen, 15_000.0);
    }

    #[test]
    fn test_aggregate_risk_metrics_empty() {
        let equity = 300_000.0;
        let positions = vec![];
        let leverage = 25.0;

        let metrics = aggregate_risk_metrics(equity, &positions, leverage);

        assert_eq!(metrics.notional_value, 0.0);
        assert_eq!(metrics.required_margin, 0.0);
        assert_eq!(metrics.effective_leverage, 0.0);
        assert_eq!(metrics.margin_rate, 0.0);
        assert_eq!(metrics.loss_per_1yen, 0.0);
    }

    #[test]
    fn test_check_daily_loss_limit_not_exceeded() {
        assert!(!check_daily_loss_limit(-1000.0, 5000.0));
        assert!(!check_daily_loss_limit(-5000.0, 5000.0));
    }

    #[test]
    fn test_check_daily_loss_limit_exceeded() {
        assert!(check_daily_loss_limit(-5000.1, 5000.0));
        assert!(check_daily_loss_limit(-6000.0, 5000.0));
    }

    #[test]
    fn test_check_daily_loss_limit_profit() {
        assert!(!check_daily_loss_limit(0.0, 5000.0));
        assert!(!check_daily_loss_limit(2000.0, 5000.0));
    }

    #[test]
    fn test_check_daily_loss_limit_disabled() {
        assert!(!check_daily_loss_limit(-10000.0, 0.0));
        assert!(!check_daily_loss_limit(-10000.0, -100.0));
    }
}
