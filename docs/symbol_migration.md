# Symbol Migration Guide: String-Based to Typed `FxSymbol`

This guide documents the planned migration from error-prone, string-based symbol representation to the new type-safe `FxSymbol` enum in the `gmo-coin-fx-rs` library.

## Background
Currently, the codebase uses `String` and `&str` for currency pair symbol parameters and fields. While simple, this is prone to typos (e.g. `"USD-JPY"`, `"USDJPY"`, or `"usd_jpy"`), and lacks compile-time validation.
The introduction of `FxSymbol` provides type safety, Serde serialization support, and compile-time correctness guarantees.

---

## 1. Affected Codebase Locations

Below is a list of all currently tracked string-based symbol usages that will be migrated to `FxSymbol` in a future major version update.

### A. Core Models (`gmo-coin-fx-core`)
The following structs represent GMO Coin FX API request and response bodies. Their `symbol` fields will be migrated to `FxSymbol`:

*   **`Ticker`** in [market_models.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/market_models.rs)
    ```rust
    pub struct Ticker {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`Symbol`** in [market_models.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/market_models.rs)
    ```rust
    pub struct Symbol {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`OrderRequest`** in [order_models.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/order_models.rs)
    ```rust
    pub struct OrderRequest {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`CancelBulkOrderRequest`** in [order_models.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/order_models.rs)
    ```rust
    pub struct CancelBulkOrderRequest {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`CloseBulkOrderRequest`** in [order_models.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/order_models.rs)
    ```rust
    pub struct CloseBulkOrderRequest {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`SpeedOrderRequest`** in [order_models.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/order_models.rs)
    ```rust
    pub struct SpeedOrderRequest {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`Order`** in [order_models.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/order_models.rs)
    ```rust
    pub struct Order {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`Position`** in [position.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/position.rs)
    ```rust
    pub struct Position {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`PositionSummary`** in [position.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/position.rs)
    ```rust
    pub struct PositionSummary {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`TickerEvent`** in [ws_events.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/ws_events.rs)
    ```rust
    pub struct TickerEvent {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`ExecutionEvent`** in [ws_events.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/ws_events.rs)
    ```rust
    pub struct ExecutionEvent {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`PositionEvent`** in [ws_events.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/ws_events.rs)
    ```rust
    pub struct PositionEvent {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`OrderEvent`** in [ws_events.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/ws_events.rs)
    ```rust
    pub struct OrderEvent {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```

### B. Client Gateway and Policies (`gmo-coin-fx-client`)
*   **`SpreadThreshold`** and **`SpreadEvaluation`** in [spread_guard.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/client/src/policy/spread_guard.rs)
    ```rust
    pub struct SpreadThreshold {
        pub symbol: FxSymbol, // Migrating from String
        ...
    }
    ```
*   **`GmoFxGateway`** REST methods in [gmo_fx_gateway.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/client/src/gateway/gmo_fx_gateway.rs):
    - `klines(symbol: &str, ...)` -> `klines(symbol: FxSymbol, ...)`
    - `positions(symbol: Option<&str>, ...)` -> `positions(symbol: Option<FxSymbol>, ...)`
    - `order(symbol: Option<&str>, ...)` -> `order(symbol: Option<FxSymbol>, ...)`
    - `position_summary(symbol: Option<&str>, ...)` -> `position_summary(symbol: Option<FxSymbol>, ...)`

### C. WebSocket Client (`gmo-coin-fx-ws`)
*   **`SubscribeCommand`** in [ws.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/core/src/models/ws.rs):
    ```rust
    pub struct SubscribeCommand {
        pub symbol: Option<FxSymbol>, // Migrating from Option<String>
        ...
    }
    ```
*   **`subscribe`** in [public.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/ws/src/public.rs):
    - `subscribe(channel: &str, symbol: Option<&str>)` -> `subscribe(channel: &str, symbol: Option<FxSymbol>)`

### D. Domain Calculations (`gmo-coin-fx-domain-risk`)
*   **`pip_size`** in [position_size.rs](file:///home/hirontan/work/gmo-coin-fx-rs/crates/domain-risk/src/position_size.rs):
    - `pip_size(symbol: &str) -> f64` -> `pip_size(symbol: FxSymbol) -> f64`

---

## 2. Migration Examples

### Example: Checking Pip Size
Before:
```rust
use gmo_coin_fx_domain_risk::position_size::pip_size;

let size = pip_size("USD_JPY"); // Type: &str
```
After:
```rust
use gmo_coin_fx_core::models::FxSymbol;
use gmo_coin_fx_domain_risk::position_size::pip_size;

let size = pip_size(FxSymbol::UsdJpy); // Type-safe FxSymbol enum
```

### Example: Creating an Order
Before:
```rust
use gmo_coin_fx_core::models::{OrderRequest, OrderSide, ExecutionType};

let request = OrderRequest {
    symbol: "USD_JPY".to_string(),
    side: OrderSide::BUY,
    size: "10000".to_string(),
    execution_type: ExecutionType::MARKET,
    // ...
};
```
After:
```rust
use gmo_coin_fx_core::models::{OrderRequest, OrderSide, ExecutionType, FxSymbol};

let request = OrderRequest {
    symbol: FxSymbol::UsdJpy,
    side: OrderSide::BUY,
    size: "10000".to_string(),
    execution_type: ExecutionType::MARKET,
    // ...
};
```
