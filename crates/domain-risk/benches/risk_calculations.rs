use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gmo_coin_fx_domain_risk::{
    aggregate_risk_metrics, calculate_risk_metrics, check_daily_loss_limit, check_order_risk,
    drawdown_pct, leverage::effective_leverage, margin::margin_rate, margin::required_margin,
    pip_size, pip_value, position_size::max_quantity_by_leverage,
    position_size::max_quantity_by_risk, position_size::notional_value, position_size::risk_amount,
    position_size::round_down_to_unit, position_size::stop_distance_from_risk,
    position_size::take_profit_distance, trailing_stop_from_atr, trailing_stop_from_pct,
    RiskConfig,
};

fn bench_individual_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("individual_calculations");

    // notional_value
    group.bench_function("notional_value", |b| {
        let quantity = black_box(20_000.0);
        let price = black_box(157.56);
        b.iter(|| notional_value(quantity, price));
    });

    // required_margin
    group.bench_function("required_margin", |b| {
        let quantity = black_box(20_000.0);
        let price = black_box(157.56);
        let leverage = black_box(25.0);
        b.iter(|| required_margin(quantity, price, leverage));
    });

    // effective_leverage
    group.bench_function("effective_leverage", |b| {
        let quantity = black_box(20_000.0);
        let price = black_box(157.56);
        let equity = black_box(300_000.0);
        b.iter(|| effective_leverage(quantity, price, equity));
    });

    // margin_rate
    group.bench_function("margin_rate", |b| {
        let equity = black_box(300_000.0);
        let req_margin = black_box(126_048.0);
        b.iter(|| margin_rate(equity, req_margin));
    });

    // drawdown_pct
    group.bench_function("drawdown_pct", |b| {
        let peak = black_box(350_000.0);
        let current = black_box(300_000.0);
        b.iter(|| drawdown_pct(peak, current));
    });

    // risk_amount
    group.bench_function("risk_amount", |b| {
        let equity = black_box(300_000.0);
        let risk_pct = black_box(0.02);
        b.iter(|| risk_amount(equity, risk_pct));
    });

    // max_quantity_by_risk
    group.bench_function("max_quantity_by_risk", |b| {
        let equity = black_box(300_000.0);
        let risk_pct = black_box(0.02);
        let stop_distance = black_box(0.5);
        b.iter(|| max_quantity_by_risk(equity, risk_pct, stop_distance));
    });

    // stop_distance_from_risk
    group.bench_function("stop_distance_from_risk", |b| {
        let equity = black_box(300_000.0);
        let risk_pct = black_box(0.02);
        let quantity = black_box(12_000.0);
        b.iter(|| stop_distance_from_risk(equity, risk_pct, quantity));
    });

    // take_profit_distance
    group.bench_function("take_profit_distance", |b| {
        let stop_distance = black_box(0.5);
        let rr_ratio = black_box(2.0);
        b.iter(|| take_profit_distance(stop_distance, rr_ratio));
    });

    // pip_size
    group.bench_function("pip_size", |b| {
        let symbol = black_box("USD_JPY");
        b.iter(|| pip_size(symbol));
    });

    // pip_value
    group.bench_function("pip_value", |b| {
        let quantity = black_box(20_000.0);
        let pip = black_box(0.01);
        b.iter(|| pip_value(quantity, pip));
    });

    // max_quantity_by_leverage
    group.bench_function("max_quantity_by_leverage", |b| {
        let equity = black_box(300_000.0);
        let max_lev = black_box(10.0);
        let price = black_box(157.56);
        b.iter(|| max_quantity_by_leverage(equity, max_lev, price));
    });

    // round_down_to_unit
    group.bench_function("round_down_to_unit", |b| {
        let quantity = black_box(19_040.36);
        let unit = black_box(1_000.0);
        b.iter(|| round_down_to_unit(quantity, unit));
    });

    // trailing_stop_from_atr
    group.bench_function("trailing_stop_from_atr", |b| {
        let atr = black_box(0.45);
        let multiplier = black_box(2.0);
        b.iter(|| trailing_stop_from_atr(atr, multiplier));
    });

    // trailing_stop_from_pct
    group.bench_function("trailing_stop_from_pct", |b| {
        let price = black_box(157.56);
        let pct = black_box(0.01);
        b.iter(|| trailing_stop_from_pct(price, pct));
    });

    // check_daily_loss_limit
    group.bench_function("check_daily_loss_limit", |b| {
        let daily_pnl = black_box(-6000.0);
        let max_loss = black_box(5000.0);
        b.iter(|| check_daily_loss_limit(daily_pnl, max_loss));
    });

    group.finish();
}

fn bench_risk_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("risk_metrics");

    // calculate_risk_metrics
    group.bench_function("calculate_risk_metrics", |b| {
        let equity = black_box(300_000.0);
        let quantity = black_box(20_000.0);
        let price = black_box(157.56);
        let leverage = black_box(25.0);
        b.iter(|| calculate_risk_metrics(equity, quantity, price, leverage));
    });

    // aggregate_risk_metrics (2 positions)
    let positions_2 = black_box(vec![(10_000.0, 150.0, 0.0), (-5_000.0, 150.0, 0.0)]);
    group.bench_function("aggregate_risk_metrics_2_positions", |b| {
        let equity = black_box(300_000.0);
        let leverage = black_box(25.0);
        b.iter(|| aggregate_risk_metrics(equity, &positions_2, leverage));
    });

    // aggregate_risk_metrics (10 positions)
    let positions_10 = black_box(vec![
        (10_000.0, 150.0, 100.0),
        (-5_000.0, 150.5, -50.0),
        (20_000.0, 149.8, 200.0),
        (-10_000.0, 151.0, -120.0),
        (5_000.0, 150.2, 30.0),
        (15_000.0, 149.5, 450.0),
        (-8_000.0, 150.8, -80.0),
        (12_000.0, 150.1, 150.0),
        (-6_000.0, 150.6, -60.0),
        (25_000.0, 149.2, 750.0),
    ]);
    group.bench_function("aggregate_risk_metrics_10_positions", |b| {
        let equity = black_box(1_000_000.0);
        let leverage = black_box(25.0);
        b.iter(|| aggregate_risk_metrics(equity, &positions_10, leverage));
    });

    group.finish();
}

fn bench_order_risk(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_risk");

    let config = RiskConfig {
        max_effective_leverage: 25.0,
        min_margin_rate: 100.0,
        risk_per_trade_pct: 0.02,
        quantity_unit: 1000.0,
        max_open_positions: Some(5),
    };

    // check_order_risk - allowed (without stop distance)
    group.bench_function("check_order_risk_allowed_no_stop", |b| {
        let equity = black_box(300_000.0);
        let quantity = black_box(5_000.0);
        let price = black_box(157.56);
        let leverage = black_box(25.0);
        let position_count = black_box(0);
        let cfg = black_box(config);
        b.iter(|| check_order_risk(equity, quantity, price, leverage, position_count, None, cfg));
    });

    // check_order_risk - allowed (with stop distance)
    group.bench_function("check_order_risk_allowed_with_stop", |b| {
        let equity = black_box(300_000.0);
        let quantity = black_box(5_000.0);
        let price = black_box(157.56);
        let leverage = black_box(25.0);
        let position_count = black_box(1);
        let stop_distance = black_box(Some(0.5));
        let cfg = black_box(config);
        b.iter(|| {
            check_order_risk(
                equity,
                quantity,
                price,
                leverage,
                position_count,
                stop_distance,
                cfg,
            )
        });
    });

    // check_order_risk - rejected (multiple violations)
    let strict_config = RiskConfig {
        max_effective_leverage: 5.0,
        min_margin_rate: 500.0,
        risk_per_trade_pct: 0.02,
        quantity_unit: 1000.0,
        max_open_positions: Some(2),
    };
    group.bench_function("check_order_risk_rejected", |b| {
        let equity = black_box(300_000.0);
        let quantity = black_box(20_000.0);
        let price = black_box(157.56);
        let leverage = black_box(25.0);
        let position_count = black_box(2);
        let stop_distance = black_box(Some(0.5));
        let cfg = black_box(strict_config);
        b.iter(|| {
            check_order_risk(
                equity,
                quantity,
                price,
                leverage,
                position_count,
                stop_distance,
                cfg,
            )
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_individual_calculations,
    bench_risk_metrics,
    bench_order_risk
);
criterion_main!(benches);
