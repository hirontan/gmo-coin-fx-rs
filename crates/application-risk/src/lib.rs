use gmo_coin_fx_client::GmoFxClient;
use gmo_coin_fx_core::Result;
use gmo_coin_fx_domain_risk::types::{RiskCheckResult, RiskConfig};

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
}
