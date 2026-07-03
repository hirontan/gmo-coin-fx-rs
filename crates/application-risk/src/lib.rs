use gmo_coin_fx_client::GmoFxClient;
use gmo_coin_fx_core::models::{Order, OrderRequest};
use gmo_coin_fx_core::Result;
use gmo_coin_fx_domain_risk::types::{RiskCheckResult, RiskConfig, RiskMetrics};

pub async fn evaluate_order_risk(
    client: &GmoFxClient,
    quantity: f64,
    price: f64,
    config: RiskConfig,
) -> Result<RiskCheckResult> {
    let assets = client.assets().await?;
    let asset = assets.first().ok_or_else(|| {
        gmo_coin_fx_core::error::GmoFxError::InvalidRequest(
            "No account assets returned".to_string(),
        )
    })?;
    let equity = f64::try_from(asset)?;

    let positions = client.open_positions(None, None).await?;
    let current_position_count = positions.list.len();

    let account_leverage = if let Some(first_pos) = positions.list.first() {
        first_pos.leverage_f64().unwrap_or(25.0)
    } else {
        25.0
    };

    let result = gmo_coin_fx_domain_risk::risk_guard::check_order_risk(
        equity,
        quantity,
        price,
        account_leverage,
        current_position_count,
        None, // stop_distance
        config,
    );

    Ok(result)
}

pub async fn portfolio_risk_summary(
    client: &GmoFxClient,
    _config: RiskConfig,
) -> Result<RiskMetrics> {
    let assets = client.assets().await?;
    let asset = assets.first().ok_or_else(|| {
        gmo_coin_fx_core::error::GmoFxError::InvalidRequest(
            "No account assets returned".to_string(),
        )
    })?;
    let equity = f64::try_from(asset)?;

    let positions = client.open_positions(None, None).await?;

    let mut mapped_positions = Vec::new();
    for p in &positions.list {
        let size = p.size_f64()?;
        let price = p.price_f64()?;
        let loss_gain = p.loss_gain_f64()?;
        let quantity = if p.side.to_uppercase() == "BUY" {
            size
        } else {
            -size
        };
        mapped_positions.push((quantity, price, loss_gain));
    }

    let account_leverage = if let Some(first_pos) = positions.list.first() {
        first_pos.leverage_f64().unwrap_or(25.0)
    } else {
        25.0
    };

    let metrics = gmo_coin_fx_domain_risk::aggregate_risk_metrics(
        equity,
        &mapped_positions,
        account_leverage,
    );

    Ok(metrics)
}

#[derive(Debug, Clone)]
pub struct SafeOrderResult {
    pub allowed: bool,
    pub reasons: Vec<String>,
    pub orders: Option<Vec<Order>>,
    pub metrics: RiskMetrics,
}

pub async fn safe_order(
    client: &GmoFxClient,
    req: &OrderRequest,
    config: RiskConfig,
) -> Result<SafeOrderResult> {
    let quantity = req.size.parse::<f64>().map_err(|e| {
        gmo_coin_fx_core::error::GmoFxError::InvalidRequest(format!("failed to parse size: {}", e))
    })?;

    // Determine price: limitPrice, stopPrice, or ticker price (for market orders)
    let price = if let Some(ref lp) = req.limit_price {
        lp.parse::<f64>().map_err(|e| {
            gmo_coin_fx_core::error::GmoFxError::InvalidRequest(format!(
                "failed to parse limitPrice: {}",
                e
            ))
        })?
    } else if let Some(ref sp) = req.stop_price {
        sp.parse::<f64>().map_err(|e| {
            gmo_coin_fx_core::error::GmoFxError::InvalidRequest(format!(
                "failed to parse stopPrice: {}",
                e
            ))
        })?
    } else {
        // Fetch ticker for market price
        let tickers = client.ticker().await?;
        let ticker = tickers
            .iter()
            .find(|t| t.symbol == req.symbol)
            .ok_or_else(|| {
                gmo_coin_fx_core::error::GmoFxError::InvalidRequest(format!(
                    "Ticker not found for symbol: {}",
                    req.symbol
                ))
            })?;
        if req.side == gmo_coin_fx_core::models::OrderSide::BUY {
            ticker.ask_f64()?
        } else {
            ticker.bid_f64()?
        }
    };

    let check_result = evaluate_order_risk(client, quantity, price, config).await?;

    if !check_result.allowed {
        return Ok(SafeOrderResult {
            allowed: false,
            reasons: check_result.reasons,
            orders: None,
            metrics: check_result.metrics,
        });
    }

    let orders = client.order(req).await?;

    Ok(SafeOrderResult {
        allowed: true,
        reasons: Vec::new(),
        orders: Some(orders),
        metrics: check_result.metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gmo_coin_fx_domain_risk::types::RiskConfig;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn start_mock_server() -> (tokio::net::TcpListener, String) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}", port);
        (listener, url)
    }

    async fn handle_connection(mut stream: tokio::net::TcpStream, body: &str) {
        let mut buf = [0; 1024];
        let _ = stream.read(&mut buf).await;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;
    }

    #[tokio::test]
    async fn test_evaluate_order_risk_success() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            // Assets request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{
                    "status": 0,
                    "data": [
                        {
                            "equity": "300000.00",
                            "availableAmount": "250000.00",
                            "balance": "300000.00",
                            "estimatedTradeFee": "0.00",
                            "margin": "50000.00",
                            "marginRatio": "500.00",
                            "positionLossGain": "0.00",
                            "totalSwap": "0.00",
                            "transferableAmount": "200000.00"
                        }
                    ]
                }"#;
                handle_connection(stream, body).await;
            }
            // Open positions request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{"status": 0, "data": {"list": []}}"#;
                handle_connection(stream, body).await;
            }
        });

        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .base_url(&url)
            .build();

        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        // Smaller quantity so effective leverage is within limits: 5000 * 150 / 300000 = 2.5x (Limit: 5.0x)
        let result = evaluate_order_risk(&client, 5000.0, 150.0, config)
            .await
            .unwrap();
        assert!(result.allowed);
        assert!(result.reasons.is_empty());
        assert_eq!(result.metrics.effective_leverage, 2.5);
    }

    #[tokio::test]
    async fn test_evaluate_order_risk_rejected() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            // Assets request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{
                    "status": 0,
                    "data": [
                        {
                            "equity": "300000.00",
                            "availableAmount": "250000.00",
                            "balance": "300000.00",
                            "estimatedTradeFee": "0.00",
                            "margin": "50000.00",
                            "marginRatio": "500.00",
                            "positionLossGain": "0.00",
                            "totalSwap": "0.00",
                            "transferableAmount": "200000.00"
                        }
                    ]
                }"#;
                handle_connection(stream, body).await;
            }
            // Open positions request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{"status": 0, "data": {"list": []}}"#;
                handle_connection(stream, body).await;
            }
        });

        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .base_url(&url)
            .build();

        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        // Larger quantity so effective leverage exceeds limit: 20000 * 150 / 300000 = 10x (Limit: 5.0x)
        let result = evaluate_order_risk(&client, 20000.0, 150.0, config)
            .await
            .unwrap();
        assert!(!result.allowed);
        assert_eq!(result.reasons.len(), 1);
        assert!(result.reasons[0].contains("Effective leverage exceeds limit"));
    }

    #[tokio::test]
    async fn test_evaluate_order_risk_empty_assets() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{"status": 0, "data": []}"#;
                handle_connection(stream, body).await;
            }
        });

        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .base_url(&url)
            .build();

        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        let result = evaluate_order_risk(&client, 5000.0, 150.0, config).await;
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("No account assets returned"));
    }

    #[tokio::test]
    async fn test_portfolio_risk_summary_success() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            // Assets request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{
                    "status": 0,
                    "data": [
                        {
                            "equity": "300000.00",
                            "availableAmount": "250000.00",
                            "balance": "300000.00",
                            "estimatedTradeFee": "0.00",
                            "margin": "50000.00",
                            "marginRatio": "500.00",
                            "positionLossGain": "0.00",
                            "totalSwap": "0.00",
                            "transferableAmount": "200000.00"
                        }
                    ]
                }"#;
                handle_connection(stream, body).await;
            }
            // Open positions request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{
                    "status": 0,
                    "data": {
                        "list": [
                            {
                                "positionId": 1234567,
                                "symbol": "USD_JPY",
                                "side": "BUY",
                                "size": "10000",
                                "orderdSize": "0",
                                "price": "150.0",
                                "lossGain": "1000",
                                "leverage": "25",
                                "losscutPrice": "130.0",
                                "timestamp": "2019-03-19T02:15:06.064Z"
                            },
                            {
                                "positionId": 1234568,
                                "symbol": "USD_JPY",
                                "side": "SELL",
                                "size": "5000",
                                "orderdSize": "0",
                                "price": "150.0",
                                "lossGain": "-500",
                                "leverage": "25",
                                "losscutPrice": "170.0",
                                "timestamp": "2019-03-19T02:15:06.064Z"
                            }
                        ]
                    }
                }"#;
                handle_connection(stream, body).await;
            }
        });

        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .base_url(&url)
            .build();

        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        let metrics = portfolio_risk_summary(&client, config).await.unwrap();

        assert_eq!(metrics.notional_value, 2_250_000.0);
        assert_eq!(metrics.required_margin, 90_000.0);
        assert_eq!(metrics.effective_leverage, 7.5);
    }

    #[tokio::test]
    async fn test_safe_order_success() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            // 1. Assets request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{
                    "status": 0,
                    "data": [
                        {
                            "equity": "300000.00",
                            "availableAmount": "250000.00",
                            "balance": "300000.00",
                            "estimatedTradeFee": "0.00",
                            "margin": "50000.00",
                            "marginRatio": "500.00",
                            "positionLossGain": "0.00",
                            "totalSwap": "0.00",
                            "transferableAmount": "200000.00"
                        }
                    ]
                }"#;
                handle_connection(stream, body).await;
            }
            // 2. Open positions request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{"status": 0, "data": {"list": []}}"#;
                handle_connection(stream, body).await;
            }
            // 3. Place order request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{
                    "status": 0,
                    "data": [
                        {
                            "rootOrderId": 12345,
                            "clientOrderId": "abc",
                            "orderId": 12345,
                            "symbol": "USD_JPY",
                            "side": "BUY",
                            "orderType": "LIMIT",
                            "executionType": "LIMIT",
                            "settleType": "OPEN",
                            "size": "5000",
                            "price": "150.0",
                            "status": "ORDERED",
                            "timestamp": "2026-06-14T22:00:00Z"
                        }
                    ]
                }"#;
                handle_connection(stream, body).await;
            }
        });

        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .base_url(&url)
            .build();

        let req = OrderRequest::builder()
            .symbol("USD_JPY")
            .side(gmo_coin_fx_core::models::OrderSide::BUY)
            .size("5000")
            .execution_type(gmo_coin_fx_core::models::ExecutionType::LIMIT)
            .limit_price("150.0")
            .build()
            .unwrap();

        let config = RiskConfig {
            max_effective_leverage: 5.0,
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        let result = safe_order(&client, &req, config).await.unwrap();

        assert!(result.allowed);
        assert!(result.reasons.is_empty());
        assert!(result.orders.is_some());
        let orders = result.orders.unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order_id, 12345);
        assert_eq!(result.metrics.effective_leverage, 2.5);
    }

    #[tokio::test]
    async fn test_safe_order_rejected() {
        let (listener, url) = start_mock_server().await;

        tokio::spawn(async move {
            // 1. Assets request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{
                    "status": 0,
                    "data": [
                        {
                            "equity": "300000.00",
                            "availableAmount": "250000.00",
                            "balance": "300000.00",
                            "estimatedTradeFee": "0.00",
                            "margin": "50000.00",
                            "marginRatio": "500.00",
                            "positionLossGain": "0.00",
                            "totalSwap": "0.00",
                            "transferableAmount": "200000.00"
                        }
                    ]
                }"#;
                handle_connection(stream, body).await;
            }
            // 2. Open positions request
            if let Ok((stream, _)) = listener.accept().await {
                let body = r#"{"status": 0, "data": {"list": []}}"#;
                handle_connection(stream, body).await;
            }
        });

        let client = GmoFxClient::builder()
            .credentials("api_key", "secret_key")
            .base_url(&url)
            .build();

        let req = OrderRequest::builder()
            .symbol("USD_JPY")
            .side(gmo_coin_fx_core::models::OrderSide::BUY)
            .size("20000") // 20000 * 150 / 300000 = 10x effective leverage
            .execution_type(gmo_coin_fx_core::models::ExecutionType::LIMIT)
            .limit_price("150.0")
            .build()
            .unwrap();

        let config = RiskConfig {
            max_effective_leverage: 5.0, // limit is 5.0x
            min_margin_rate: 200.0,
            risk_per_trade_pct: 0.02,
            quantity_unit: 1000.0,
            max_open_positions: None,
        };

        let result = safe_order(&client, &req, config).await.unwrap();

        assert!(!result.allowed);
        assert_eq!(result.reasons.len(), 1);
        assert!(result.reasons[0].contains("Effective leverage exceeds limit"));
        assert!(result.orders.is_none());
        assert_eq!(result.metrics.effective_leverage, 10.0);
    }
}
