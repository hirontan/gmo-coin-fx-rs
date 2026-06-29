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
        gmo_coin_fx_core::error::GmoFxError::InvalidRequest("No account assets returned".to_string())
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
