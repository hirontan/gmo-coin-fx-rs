# FX Risk Calculation & Pre-Order Risk Guard

This document details the FX risk formulas, calculations, and the pre-order risk validation mechanism (`RiskGuard`) used in the `gmo-coin-fx-rs` library.

---

## Core Formulas

The `domain-risk` crate implements the mathematical models used to evaluate FX trading risks. Below are the formulas implemented in the codebase:

### 1. Position Total Value (Notional Value)
The total evaluation value of a position in the quote currency (e.g., JPY for USD/JPY).
$$\text{Notional Value} = \text{Quantity} \times \text{Price}$$

### 2. Required Margin
The minimum amount of collateral required to open and maintain the position.
$$\text{Required Margin} = \frac{\text{Quantity} \times \text{Price}}{\text{Account Leverage}}$$

### 3. Effective Leverage
The actual leverage of the account based on the total position value and the current equity.
$$\text{Effective Leverage} = \frac{\text{Notional Value}}{\text{Equity}}$$

### 4. Margin Maintenance Rate
The percentage representing the current health of the margin account.
$$\text{Margin Maintenance Rate (\%)} = \frac{\text{Equity}}{\text{Required Margin}} \times 100$$

### 5. Drawdown Percentage
The percentage decline of the account equity from its historical peak value.
$$\text{Drawdown (\%)} = \frac{\text{Peak Equity} - \text{Current Equity}}{\text{Peak Equity}} \times 100$$
If the current equity is greater than or equal to the peak equity, the drawdown is $0\%$.

### 6. Adverse Movement Loss (1 Yen adverse change)
The estimated loss amount if the price moves against the position by $1$ quote unit (e.g. 1 Yen for cross-yen pairs).
$$\text{Loss Per 1 Yen} = \text{Quantity}$$

### 7. Position Sizing
* **Max Quantity by Risk**: The maximum quantity that can be traded based on the maximum risk amount (derived from equity and trade risk percentage) and the stop-loss distance.
  $$\text{Max Quantity by Risk} = \frac{\text{Equity} \times \text{Risk Per Trade (\%)}}{\text{Stop Distance}}$$
* **Stop Distance from Risk**: The stop-loss price distance based on equity, risk percentage, and quantity (the inverse of *Max Quantity by Risk*).
  $$\text{Stop Distance} = \frac{\text{Equity} \times \text{Risk Per Trade (\%)}}{\text{Quantity}}$$
* **Take-Profit Distance**: The take-profit price distance calculated from stop distance and risk-reward ratio.
  $$\text{Take-Profit Distance} = \text{Stop Distance} \times \text{Risk-Reward Ratio}$$
* **Max Quantity by Leverage**: The maximum quantity that can be traded while staying within a target maximum effective leverage limit.
  $$\text{Max Quantity by Leverage} = \frac{\text{Equity} \times \text{Max Effective Leverage}}{\text{Price}}$$
* **Round Down to Unit**: Positions must be rounded down to the nearest integer multiple of the trading unit (e.g., 1,000 units).
  $$\text{Rounded Quantity} = \lfloor\frac{\text{Quantity}}{\text{Unit}}\rfloor \times \text{Unit}$$
* **Pip Size**: The minimum standard price increment of a symbol. JPY-based pairs use $0.01$, while non-JPY pairs use $0.0001$.
* **Pip Value**: The quote currency value of a single pip move for a given quantity.
  $$\text{Pip Value} = |\text{Quantity}| \times \text{Pip Size}$$
* **Trailing Stop from ATR**: The trailing stop distance calculated using Average True Range (ATR) and a multiplier.
  $$\text{Trailing Stop Distance (ATR)} = \text{ATR} \times \text{Multiplier}$$
* **Trailing Stop from Percentage**: The trailing stop distance calculated as a percentage of the current price.
  $$\text{Trailing Stop Distance (\%)} = \text{Price} \times \text{Percentage}$$


---

## Concrete Example

Consider the following trading scenario for USD/JPY:
* **Equity**: $300,000$ JPY
* **Quantity**: $20,000$ USD
* **Price**: $157.56$ USD/JPY
* **Account Leverage**: $25$

Using the formulas, the calculated risk metrics are:

| Metric | Calculation | Result |
| :--- | :--- | :--- |
| **Position Total Value** | $20,000 \times 157.56$ | **$3,151,200$ JPY** |
| **Required Margin** | $\frac{3,151,200}{25}$ | **$126,048$ JPY** |
| **Effective Leverage** | $\frac{3,151,200}{300,000}$ | **$10.504$x** |
| **Margin Maintenance Rate** | $\frac{300,000}{126,048} \times 100$ | **$238.00$\%** (approximately **$237.99$\%**) |
| **Loss Per 1 Yen** | $20,000$ | **$20,000$ JPY** |

---

## Pre-Order Risk Guard (`RiskGuard`)

### Role and Responsibility
The **Risk Guard** acts as a safety-oriented check mechanism. Before placing any order, the system checks whether the proposed order can be safely executed under the configured risk constraints:
1. Basic input sanity: checks that `quantity > 0`, `equity > 0`, `price > 0`, `account_leverage > 0`, and when `stop_distance` is provided, `stop_distance > 0`.
2. Threshold validations: checks that the resulting `Effective Leverage` does not exceed the allowed maximum, the resulting `Margin Maintenance Rate` does not drop below the minimum threshold, and the `current_position_count` does not meet or exceed the `max_open_positions` limit.
3. Per-trade risk validation: when `stop_distance` is provided, validates that the potential loss (`quantity * stop_distance`) does not exceed the risk per trade limit (`equity * config.risk_per_trade_pct`).

> [!IMPORTANT]
> **Risk Guard does not generate or decide trading signals.**
> All trading signals (buy/sell decisions, entry/exit levels) must be generated by separate trading strategy logic. The Risk Guard acts solely as a passive pre-order filter to prevent the execution of orders that would violate safety thresholds and expose the account to unacceptable margin call or liquidation risks.

### Code Usage

You can check whether an order is safe to place by calling `check_order_risk`:

```rust
use gmo_coin_fx_domain_risk::{check_order_risk, RiskConfig};

fn verify_order_safety() {
    let config = RiskConfig {
        max_effective_leverage: 5.0,
        min_margin_rate: 500.0,
        risk_per_trade_pct: 0.02,
        quantity_unit: 1000.0,
        max_open_positions: Some(5),
    };

    let result = check_order_risk(
        300_000.0, // Equity
        20_000.0,  // Quantity
        157.56,    // Price
        25.0,      // Account Leverage
        2,         // Current Position Count
        Some(0.5), // Stop Distance (optional)
        config,
    );

    if !result.allowed {
        println!("Order rejected due to risk constraints:");
        for reason in result.reasons {
            println!(" - {}", reason);
        }
    }
}
```

**Rejection Output Example:**
```json
{
  "allowed": false,
  "reasons": [
    "Effective leverage exceeds limit: 10.5x > 5.0x",
    "Margin maintenance rate is below threshold: 238% < 500%",
    "Potential loss exceeds risk per trade limit: 10000.00 > 6000.00"
  ]
}
```

### Human-Readable Formatting (Display Trait)

Both `RiskMetrics` and `RiskCheckResult` implement the `Display` trait for human-readable output (e.g. for user UI or console logs).

**Example output for `RiskMetrics`:**
```
Position Value: ¥3,151,200 | Required Margin: ¥126,048
Effective Leverage: 10.5x | Margin Rate: 238.0%
Loss per 1¥: ¥20,000
```

**Example output for `RiskCheckResult`:**
```
Status: Rejected
Reasons:
 - Effective leverage exceeds limit: 10.5x > 5.0x
 - Margin maintenance rate is below threshold: 238% < 500%
Position Value: ¥3,151,200 | Required Margin: ¥126,048
Effective Leverage: 10.5x | Margin Rate: 238.0%
Loss per 1¥: ¥20,000
```

---

## Portfolio-Level Risk Aggregation

When holding multiple positions, portfolio-level risk metrics are calculated by aggregating individual positions:

1. **Total Notional Value**: The sum of the absolute notional values of all open positions.
   $$\text{Total Notional Value} = \sum \left( |\text{Quantity}_i| \times \text{Price}_i \right)$$
2. **Total Required Margin**: The sum of the required margins of all open positions.
   $$\text{Total Required Margin} = \sum \text{Required Margin}_i$$
3. **Overall Effective Leverage**: Calculated based on the total notional value and the current equity.
   $$\text{Overall Effective Leverage} = \frac{\text{Total Notional Value}}{\text{Equity}}$$
4. **Overall Margin Maintenance Rate**:
   $$\text{Overall Margin Maintenance Rate (\%)} = \frac{\text{Equity}}{\text{Total Required Margin}} \times 100$$
5. **Total Loss Per 1 Yen**: The sum of the absolute adverse movement losses across all open positions.
   $$\text{Total Loss Per 1 Yen} = \sum |\text{Quantity}_i|$$

### Code Usage

```rust
use gmo_coin_fx_domain_risk::aggregate_risk_metrics;

fn check_portfolio_risk() {
    let equity = 300_000.0;
    let leverage = 25.0;
    
    // List of open positions: (quantity, price, unrealized_pnl)
    let positions = vec![
        (10_000.0, 150.0, 0.0),  // Long 10,000 USD/JPY
        (-5_000.0, 150.0, 0.0),  // Short 5,000 USD/JPY
    ];

    let metrics = aggregate_risk_metrics(equity, &positions, leverage);
    println!("Total Notional Value: {} JPY", metrics.notional_value); // 2,250,000 JPY
    println!("Overall Effective Leverage: {}x", metrics.effective_leverage); // 7.5x
}
```

---

## Daily Loss Limit Guard

To prevent catastrophic drawdowns under automated trading strategies, the daily realized loss should be checked against a configured maximum limit:

- **Daily Loss Check**: If the cumulative daily realized loss exceeds `max_daily_loss`, any new order placements must be halted.
- **Formula**:
  $$\text{Loss Limit Exceeded} = (\text{Daily Realized PnL} < 0) \land (|\text{Daily Realized PnL}| > \text{Max Daily Loss})$$

### Code Usage

```rust
use gmo_coin_fx_domain_risk::check_daily_loss_limit;

fn verify_trading_halt() {
    let daily_pnl = -6000.0;    // Realized loss of 6,000 JPY
    let max_loss = 5000.0;      // Limit of 5,000 JPY

    if check_daily_loss_limit(daily_pnl, max_loss) {
        println!("Trading halted: daily loss limit exceeded.");
    } else {
        println!("Trading allowed.");
    }
}
```

---

## Performance Benchmarks & Baseline Numbers

In high-frequency trading (HFT) and algorithmic trading systems, risk calculation routines may be evaluated hundreds or thousands of times per second. To prevent performance regressions and guarantee real-time throughput, micro-benchmarks are implemented using [`criterion`](https://github.com/bheisler/criterion.rs).

### Running Benchmarks

```bash
# Run full criterion benchmark suite with statistical reporting
cargo bench -p gmo-coin-fx-domain-risk

# Run fast sanity check on benchmark harnesses (1 iteration per benchmark)
cargo test --benches
```

### Baseline Performance Numbers

> **Environment**: x86_64 Linux, Release profile (`opt-level = 3`, Rust 1.86+ / 2021 edition).

#### 1. Individual Risk Calculations (`individual_calculations`)

| Function | Median Latency | Estimated Throughput | Description |
| :--- | :--- | :--- | :--- |
| `notional_value` | ~1.02 ns | ~980M ops/s | Position notional value (`qty * price`) |
| `required_margin` | ~0.99 ns | ~1.01B ops/s | Required collateral (`qty * price / lev`) |
| `effective_leverage` | ~0.97 ns | ~1.03B ops/s | Effective leverage (`notional / equity`) |
| `margin_rate` | ~0.98 ns | ~1.02B ops/s | Margin maintenance rate (`equity / req * 100`) |
| `drawdown_pct` | ~0.99 ns | ~1.01B ops/s | Peak-to-current drawdown percentage |
| `risk_amount` | ~0.99 ns | ~1.01B ops/s | Risk capital allocation (`equity * risk%`) |
| `max_quantity_by_risk` | ~0.98 ns | ~1.02B ops/s | Position size based on stop distance |
| `stop_distance_from_risk` | ~0.99 ns | ~1.01B ops/s | Stop distance derived from trade risk |
| `take_profit_distance` | ~0.97 ns | ~1.03B ops/s | Take profit distance (`stop * rr_ratio`) |
| `pip_size` | ~33.5 ns | ~30M ops/s | Pip unit resolution from currency symbol |
| `pip_value` | ~0.50 ns | ~2.00B ops/s | Quote currency pip value (`qty * pip_size`) |
| `max_quantity_by_leverage` | ~0.99 ns | ~1.01B ops/s | Max units subject to leverage constraint |
| `round_down_to_unit` | ~0.99 ns | ~1.01B ops/s | Integer lot-unit rounding |
| `trailing_stop_from_atr` | ~0.99 ns | ~1.01B ops/s | Volatility-based trailing stop distance |
| `trailing_stop_from_pct` | ~0.99 ns | ~1.01B ops/s | Price percentage trailing stop distance |
| `check_daily_loss_limit` | ~0.50 ns | ~2.00B ops/s | Daily cumulative loss threshold check |

#### 2. Risk Metrics & Multi-Position Aggregation (`risk_metrics`)

| Function | Median Latency | Estimated Throughput | Description |
| :--- | :--- | :--- | :--- |
| `calculate_risk_metrics` | ~7.36 ns | ~135M ops/s | Single-order composite risk calculation |
| `aggregate_risk_metrics` (2 positions) | ~9.82 ns | ~102M ops/s | Portfolio-wide risk aggregation (2 legs) |
| `aggregate_risk_metrics` (10 positions) | ~31.6 ns | ~31.6M ops/s | Portfolio-wide risk aggregation (10 legs) |

#### 3. Pre-Order Risk Validation (`order_risk`)

| Scenario | Median Latency | Estimated Throughput | Description |
| :--- | :--- | :--- | :--- |
| `check_order_risk` (allowed, no stop) | ~24.8 ns | ~40.3M ops/s | Full pre-order validation without stop validation |
| `check_order_risk` (allowed, with stop) | ~25.9 ns | ~38.6M ops/s | Full pre-order validation including potential loss check |
| `check_order_risk` (rejected) | ~1.50 µs | ~667K ops/s | Validation rejection with failure reason string allocations |

### Key Takeaways
- **Ultra-low latency for hot paths**: All primary arithmetic operations execute in sub-nanosecond or single-digit nanoseconds (~1 ns).
- **Zero allocation for allowed orders**: Allowed order checks take ~25 ns and perform zero heap allocations.
- **Scalable portfolio aggregation**: Aggregating across 10 positions takes just ~32 ns, comfortably supporting hundreds of thousands of evaluation ticks per second.



