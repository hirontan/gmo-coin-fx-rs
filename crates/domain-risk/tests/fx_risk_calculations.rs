use gmo_coin_fx_domain_risk::leverage::effective_leverage;
use gmo_coin_fx_domain_risk::margin::{margin_rate, required_margin};
use gmo_coin_fx_domain_risk::position_size::{max_quantity_by_leverage, max_quantity_by_risk, round_down_to_unit};
use gmo_coin_fx_domain_risk::{check_order_risk, RiskConfig};

#[test]
fn test_required_margin() {
    let qty = 20_000.0;
    let price = 157.56;
    let leverage = 25.0;
    let margin = required_margin(qty, price, leverage).unwrap();
    assert_eq!(margin, 126_048.0);
}

#[test]
fn test_effective_leverage() {
    let qty = 20_000.0;
    let price = 157.56;
    let equity = 300_000.0;
    let el = effective_leverage(qty, price, equity).unwrap();
    assert_eq!(el, 10.504);
}

#[test]
fn test_margin_rate() {
    let equity = 300_000.0;
    let required = 126_048.0;
    let rate = margin_rate(equity, required).unwrap();
    // Expected: approximately 237.99
    assert!((rate - 237.99).abs() < 0.02);
}

#[test]
fn test_max_quantity_by_risk() {
    let equity = 300_000.0;
    let risk_per_trade_pct = 0.005;
    let stop_distance = 0.50;
    let quantity = max_quantity_by_risk(equity, risk_per_trade_pct, stop_distance).unwrap();
    assert_eq!(quantity, 3_000.0);
}

#[test]
fn test_max_quantity_by_leverage() {
    let equity = 300_000.0;
    let max_effective_leverage = 3.0;
    let price = 157.56;
    let raw_qty = max_quantity_by_leverage(equity, max_effective_leverage, price).unwrap();
    // Actual calculation gives 5712.109... The issue states approximately 5,711. We check both with floating point tolerance.
    assert!((raw_qty - 5_711.0).abs() < 2.0);
    assert!((raw_qty - 5_712.109).abs() < 0.01);
}

#[test]
fn test_round_down_to_unit() {
    let quantity = 5_711.0;
    let unit = 1_000.0;
    let rounded = round_down_to_unit(quantity, unit).unwrap();
    assert_eq!(rounded, 5_000.0);
}

#[test]
fn test_risk_guard_rejection() {
    let equity = 300_000.0;
    let quantity = 20_000.0;
    let price = 157.56;
    let account_leverage = 25.0;
    let config = RiskConfig {
        max_effective_leverage: 5.0,
        min_margin_rate: 500.0,
        risk_per_trade_pct: 0.02,
        quantity_unit: 1000.0,
    };
    let result = check_order_risk(equity, quantity, price, account_leverage, config);
    assert!(!result.allowed);
    assert_eq!(result.reasons.len(), 2);
}
