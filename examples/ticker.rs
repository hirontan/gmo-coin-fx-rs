use gmo_coin_fx_client::GmoFxClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = GmoFxClient::builder().build();

    let tickers = client.ticker().await?;

    for ticker in tickers {
        println!(
            "{} bid={} ask={} status={}",
            ticker.symbol, ticker.bid, ticker.ask, ticker.status
        );
    }

    Ok(())
}
