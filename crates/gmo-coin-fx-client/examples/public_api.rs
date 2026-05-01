use gmo_coin_fx_client::GmoFxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GmoFxClient::builder().build();

    println!("Fetching status...");
    let status = client.status().await?;
    println!("Status: {:?}", status);

    println!("\nFetching symbols...");
    let symbols = client.symbols().await?;
    println!("Symbols: {:?}", symbols);

    println!("\nFetching ticker...");
    let ticker = client.ticker().await?;
    println!("Ticker: {:?}", ticker);

    println!("\nFetching klines for USD_JPY...");
    let klines = client.klines("USD_JPY", "ASK", "1min", "20231028").await?;
    println!("Klines count: {}", klines.len());

    Ok(())
}
