use gmo_coin_fx_domain_risk::leverage::effective_leverage;
use gmo_coin_fx_domain_risk::margin::{drawdown_pct, margin_rate, required_margin};
use gmo_coin_fx_domain_risk::position_size::{
    max_quantity_by_leverage, max_quantity_by_risk, notional_value, risk_amount,
    round_down_to_unit, stop_distance_from_risk, take_profit_distance, trailing_stop_from_atr,
    trailing_stop_from_pct,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_notional_value(
        quantity in 0.0001f64..1_000_000.0,
        price in 0.0001f64..1_000_000.0
    ) {
        let res = notional_value(quantity, price).unwrap();
        let expected = quantity * price;
        prop_assert!((res - expected).abs() < 1e-9);
    }

    #[test]
    fn prop_risk_amount(
        equity in 0.0001f64..1_000_000.0,
        risk_pct in 0.0001f64..1.0
    ) {
        let res = risk_amount(equity, risk_pct).unwrap();
        let expected = equity * risk_pct;
        prop_assert!((res - expected).abs() < 1e-9);
    }

    #[test]
    fn prop_max_quantity_by_risk(
        equity in 1.0f64..1_000_000.0,
        risk_pct in 0.0001f64..1.0,
        stop_distance in 0.0001f64..1_000.0
    ) {
        let qty = max_quantity_by_risk(equity, risk_pct, stop_distance).unwrap();
        let amt = risk_amount(equity, risk_pct).unwrap();
        let expected = amt / stop_distance;
        prop_assert!((qty - expected).abs() < 1e-9);
    }

    #[test]
    fn prop_stop_distance_from_risk(
        equity in 1.0f64..1_000_000.0,
        risk_pct in 0.0001f64..1.0,
        quantity in 0.0001f64..1_000_000.0
    ) {
        let dist = stop_distance_from_risk(equity, risk_pct, quantity).unwrap();
        let amt = risk_amount(equity, risk_pct).unwrap();
        let expected = amt / quantity;
        prop_assert!((dist - expected).abs() < 1e-9);
    }

    #[test]
    fn prop_take_profit_distance(
        stop_distance in 0.0001f64..1_000.0,
        risk_reward_ratio in 0.0001f64..100.0
    ) {
        let dist = take_profit_distance(stop_distance, risk_reward_ratio).unwrap();
        let expected = stop_distance * risk_reward_ratio;
        prop_assert!((dist - expected).abs() < 1e-9);
    }

    #[test]
    fn prop_max_quantity_by_leverage(
        equity in 1.0f64..1_000_000.0,
        max_leverage in 0.0001f64..100.0,
        price in 0.0001f64..1_000_000.0
    ) {
        let qty = max_quantity_by_leverage(equity, max_leverage, price).unwrap();
        let expected = (equity * max_leverage) / price;
        prop_assert!((qty - expected).abs() < 1e-9);
    }

    #[test]
    fn prop_round_down_to_unit(
        quantity in 0.0f64..1_000_000.0,
        unit in 0.0001f64..100_000.0
    ) {
        let rounded = round_down_to_unit(quantity, unit).unwrap();
        prop_assert!(rounded <= quantity);

        let remainder = rounded / unit;
        let expected_remainder = remainder.round();
        prop_assert!((remainder - expected_remainder).abs() < 1e-9);
    }

    #[test]
    fn prop_trailing_stop_from_atr(
        atr in 0.0001f64..1_000.0,
        multiplier in 0.0001f64..100.0
    ) {
        let dist = trailing_stop_from_atr(atr, multiplier).unwrap();
        let expected = atr * multiplier;
        prop_assert!((dist - expected).abs() < 1e-9);
    }

    #[test]
    fn prop_trailing_stop_from_pct(
        price in 0.0001f64..1_000_000.0,
        pct in 0.0001f64..1.0
    ) {
        let dist = trailing_stop_from_pct(price, pct).unwrap();
        let expected = price * pct;
        prop_assert!((dist - expected).abs() < 1e-9);
    }

    #[test]
    fn prop_effective_leverage_positive(
        quantity in 0.0001f64..1_000_000.0,
        price in 0.0001f64..1_000_000.0,
        equity in 0.0001f64..1_000_000.0
    ) {
        let el = effective_leverage(quantity, price, equity).unwrap();
        prop_assert!(el > 0.0);
    }

    #[test]
    fn prop_margin_rate_inversely_proportional_to_leverage(
        equity in 1.0f64..1_000_000.0,
        price in 1.0f64..10_000.0,
        leverage in 1.0f64..100.0,
        q1 in 1.0f64..10_000.0,
        q2 in 1.0f64..10_000.0
    ) {
        let (qty_small, qty_large) = if q1 < q2 {
            (q1, q2)
        } else if q1 > q2 {
            (q2, q1)
        } else {
            (q1, q1 + 1.0)
        };

        let el_small = effective_leverage(qty_small, price, equity).unwrap();
        let el_large = effective_leverage(qty_large, price, equity).unwrap();
        prop_assert!(el_small < el_large);

        let req_margin_small = required_margin(qty_small, price, leverage).unwrap();
        let req_margin_large = required_margin(qty_large, price, leverage).unwrap();
        prop_assert!(req_margin_small < req_margin_large);

        let rate_small = margin_rate(equity, req_margin_small).unwrap();
        let rate_large = margin_rate(equity, req_margin_large).unwrap();
        prop_assert!(rate_small > rate_large);
    }

    #[test]
    fn prop_drawdown_pct(
        peak in 0.0001f64..1_000_000.0,
        current in 0.0f64..1_000_000.0
    ) {
        let draw = drawdown_pct(peak, current).unwrap();
        prop_assert!(draw >= 0.0);
        prop_assert!(draw <= 100.0);

        if current >= peak {
            prop_assert_eq!(draw, 0.0);
        } else {
            let expected = ((peak - current) / peak) * 100.0;
            prop_assert!((draw - expected).abs() < 1e-9);
        }
    }
}
