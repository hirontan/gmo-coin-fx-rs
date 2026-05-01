use gmo_coin_fx_client::GmoFxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GMO_API_KEY").unwrap_or_else(|_| "DUMMY_KEY".to_string());
    let secret_key = std::env::var("GMO_SECRET_KEY").unwrap_or_else(|_| "DUMMY_SECRET".to_string());

    let client = GmoFxClient::builder()
        .credentials(api_key, secret_key)
        .build();

    println!("Fetching account assets...");
    match client.assets().await {
        Ok(assets) => println!("Assets: {:?}", assets),
        Err(e) => println!("API Error (expected if dummy credentials): {:?}", e),
    }

    println!("\nFetching open positions for USD_JPY...");
    match client.open_positions(Some("USD_JPY"), None).await {
        Ok(positions) => println!("Positions: {:?}", positions.list),
        Err(e) => println!("API Error (expected if dummy credentials): {:?}", e),
    }

    Ok(())
}
