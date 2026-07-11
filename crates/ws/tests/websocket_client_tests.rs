use futures_util::{SinkExt, StreamExt};
use gmo_coin_fx_client::auth::AuthSigner;
use gmo_coin_fx_client::GmoFxClient;
use gmo_coin_fx_core::models::ws::{Channel, Subscription};
use gmo_coin_fx_core::models::ws_events::{PrivateWsMessage, PublicWsMessage};
use gmo_coin_fx_core::models::FxSymbol;
use gmo_coin_fx_ws::{PrivateWsClient, PublicWsClient, ReconnectConfig};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_public_ws_reconnect_and_restore() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}", addr);

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let tx_clone = tx.clone();

    // Spawns first mock server instance
    let server_task = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let mut ws_stream = accept_async(stream).await.unwrap();
            // Expect subscription request
            if let Some(Ok(Message::Text(txt))) = ws_stream.next().await {
                tx_clone.send(format!("conn1:{}", txt)).await.unwrap();
            }
            // Send ticker event
            let ticker_json = r#"{"symbol":"USD_JPY","ask":"150.25","bid":"150.20","timestamp":"2026-07-11T12:00:00Z","status":"OPEN"}"#;
            ws_stream
                .send(Message::Text(ticker_json.into()))
                .await
                .unwrap();

            // Force disconnect by dropping ws_stream after a short sleep
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    let reconnect_config = ReconnectConfig::default()
        .initial_delay(Duration::from_millis(50))
        .max_delay(Duration::from_millis(100));

    let mut client = PublicWsClient::builder()
        .url(&url)
        .ping_interval(Duration::from_secs(10))
        .reconnect_config(reconnect_config)
        .connect()
        .await
        .unwrap();

    client
        .subscribe(
            Subscription::builder()
                .channel(Channel::Ticker)
                .symbol(FxSymbol::UsdJpy)
                .build(),
        )
        .await
        .unwrap();

    // Verify first connection subscription & message
    let sub1 = rx.recv().await.unwrap();
    assert!(sub1.contains("conn1:"));
    assert!(sub1.contains("USD_JPY"));

    let msg1 = client.next_message().await.unwrap();
    assert!(msg1.is_some());
    if let Some(PublicWsMessage::Ticker(t)) = msg1 {
        assert_eq!(t.symbol, "USD_JPY");
        assert_eq!(t.ask, "150.25");
    } else {
        panic!("Expected Ticker event");
    }

    // Server drops connection. Wait for server task to finish.
    server_task.await.unwrap();

    // Start second server instance on the same address to receive the reconnect
    let listener_reconnect = TcpListener::bind(addr).await.unwrap();
    let tx_clone2 = tx.clone();
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener_reconnect.accept().await {
            let mut ws_stream = accept_async(stream).await.unwrap();
            // The client must automatically restore the subscription!
            if let Some(Ok(Message::Text(txt))) = ws_stream.next().await {
                tx_clone2.send(format!("conn2:{}", txt)).await.unwrap();
            }
            // Send another ticker event
            let ticker_json = r#"{"symbol":"USD_JPY","ask":"150.30","bid":"150.28","timestamp":"2026-07-11T12:01:00Z","status":"OPEN"}"#;
            ws_stream
                .send(Message::Text(ticker_json.into()))
                .await
                .unwrap();
        }
    });

    // Verify subscription restore on second connection
    let sub2 = rx.recv().await.unwrap();
    assert!(sub2.contains("conn2:"));
    assert!(sub2.contains("USD_JPY"));

    // Verify we receive the new message after reconnection
    let msg2 = client.next_message().await.unwrap();
    assert!(msg2.is_some());
    if let Some(PublicWsMessage::Ticker(t)) = msg2 {
        assert_eq!(t.symbol, "USD_JPY");
        assert_eq!(t.ask, "150.30");
    } else {
        panic!("Expected Ticker event after reconnect");
    }
}

#[tokio::test]
async fn test_private_ws_reconnect_and_restore() {
    // 1. Mock HTTP Server using wiremock
    let mock_http_server = wiremock::MockServer::start().await;

    let token_response = serde_json::json!({
        "status": 0,
        "messages": [],
        "data": {
            "token": "test-token-123"
        }
    });

    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/private/v1/ws-auth"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(token_response))
        .mount(&mock_http_server)
        .await;

    // 2. Mock WS Server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ws_url = format!("ws://{}", addr);

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let tx_clone = tx.clone();

    let server_task = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let mut ws_stream = accept_async(stream).await.unwrap();
            // Expect subscription
            if let Some(Ok(Message::Text(txt))) = ws_stream.next().await {
                tx_clone.send(format!("conn1:{}", txt)).await.unwrap();
            }
            // Send execution event
            let exec_json = r#"{"channel":"executionEvents","rootOrderId":123,"orderId":456,"executionId":789,"symbol":"USD_JPY","settleType":"OPEN","orderType":"NORMAL","executionType":"LIMIT","side":"BUY","executionPrice":"150.25","executionSize":"10000","positionId":999,"lossGain":"0","fee":"0","orderPrice":"150.25","orderExecutedSize":"10000","orderSize":"10000","msgType":"ER","orderTimestamp":"2026-07-11T12:00:00Z","executionTimestamp":"2026-07-11T12:00:05Z"}"#;
            ws_stream
                .send(Message::Text(exec_json.into()))
                .await
                .unwrap();

            // Force disconnect
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    let client = GmoFxClient::builder()
        .credentials("test_api_key", "test_secret_key")
        .base_url(&mock_http_server.uri())
        .build();

    let reconnect_config = ReconnectConfig::default()
        .initial_delay(Duration::from_millis(50))
        .max_delay(Duration::from_millis(100));

    let mut ws_client = PrivateWsClient::builder(client)
        .url_base(&ws_url)
        .ping_interval(Duration::from_secs(10))
        .reconnect_config(reconnect_config)
        .connect()
        .await
        .unwrap();

    ws_client
        .subscribe(
            Subscription::builder()
                .channel(Channel::ExecutionEvents)
                .build(),
        )
        .await
        .unwrap();

    // Verify first connection
    let sub1 = rx.recv().await.unwrap();
    assert!(sub1.contains("conn1:"));
    assert!(sub1.contains("executionEvents"));

    let msg1 = ws_client.next_message().await.unwrap();
    assert!(msg1.is_some());
    if let Some(PrivateWsMessage::Execution(e)) = msg1 {
        assert_eq!(e.execution_id, 789);
        assert_eq!(e.execution_price, "150.25");
    } else {
        panic!("Expected Execution event");
    }

    server_task.await.unwrap();

    // Start second WS mock server to receive reconnect
    let listener_reconnect = TcpListener::bind(addr).await.unwrap();
    let tx_clone2 = tx.clone();
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener_reconnect.accept().await {
            let mut ws_stream = accept_async(stream).await.unwrap();
            // Expect restored subscription
            if let Some(Ok(Message::Text(txt))) = ws_stream.next().await {
                tx_clone2.send(format!("conn2:{}", txt)).await.unwrap();
            }
            // Send execution event
            let exec_json = r#"{"channel":"executionEvents","rootOrderId":123,"orderId":456,"executionId":790,"symbol":"USD_JPY","settleType":"OPEN","orderType":"NORMAL","executionType":"LIMIT","side":"BUY","executionPrice":"150.30","executionSize":"10000","positionId":999,"lossGain":"0","fee":"0","orderPrice":"150.30","orderExecutedSize":"10000","orderSize":"10000","msgType":"ER","orderTimestamp":"2026-07-11T12:00:00Z","executionTimestamp":"2026-07-11T12:00:05Z"}"#;
            ws_stream
                .send(Message::Text(exec_json.into()))
                .await
                .unwrap();
        }
    });

    // Verify subscription restore on second connection
    let sub2 = rx.recv().await.unwrap();
    assert!(sub2.contains("conn2:"));
    assert!(sub2.contains("executionEvents"));

    // Verify we receive the new message after reconnection
    let msg2 = ws_client.next_message().await.unwrap();
    assert!(msg2.is_some());
    if let Some(PrivateWsMessage::Execution(e)) = msg2 {
        assert_eq!(e.execution_id, 790);
        assert_eq!(e.execution_price, "150.30");
    } else {
        panic!("Expected Execution event after reconnect");
    }
}
