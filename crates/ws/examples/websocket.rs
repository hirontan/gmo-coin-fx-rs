use gmo_coin_fx_core::models::{Channel, FxSymbol, Subscription};
use gmo_coin_fx_ws::PublicWsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Public WebSocket...");
    let mut client = PublicWsClient::connect().await?;

    println!("Subscribing to ticker channel for USD_JPY...");
    let sub = Subscription::builder()
        .channel(Channel::Ticker)
        .symbol(FxSymbol::UsdJpy)
        .build();
    client.subscribe(sub).await?;

    println!("Waiting for messages...");
    for _ in 0..5 {
        if let Some(msg) = client.next_message().await? {
            println!("Received Event: {:?}", msg);
        }
    }

    Ok(())
}
