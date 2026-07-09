use gmo_coin_fx_client::auth::AuthSigner;
use gmo_coin_fx_client::gateway::RetryConfig;
use gmo_coin_fx_client::rest::{PrivateRestClient, PublicRestClient};
use gmo_coin_fx_core::{models::ApiResponse, ApiMessage, GmoFxError};
use wiremock::matchers::{body_json, header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_public_client_success() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 0,
        messages: None,
        data: "public_success".to_string(),
        responsetime: None,
    };

    Mock::given(method("GET"))
        .and(path("/public/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let client = PublicRestClient::new(None, None, None, None, false, Some(mock_server.uri()));

    let res: Result<String, GmoFxError> = client.get("/test").await;
    assert_eq!(res.unwrap(), "public_success");
}

#[tokio::test]
async fn test_public_client_api_error() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 10002,
        messages: Some(vec![ApiMessage {
            message_code: Some("10002".to_string()),
            message_string: Some("Invalid request".to_string()),
        }]),
        data: "".to_string(),
        responsetime: None,
    };

    Mock::given(method("GET"))
        .and(path("/public/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let client = PublicRestClient::new(None, None, None, None, false, Some(mock_server.uri()));

    let res: Result<String, GmoFxError> = client.get("/test").await;
    match res {
        Err(GmoFxError::Api { status, messages }) => {
            assert_eq!(status, 10002);
            let msgs = messages.unwrap();
            assert_eq!(
                msgs[0].message_string.as_deref().unwrap(),
                "Invalid request"
            );
        }
        _ => panic!("Expected Api Error"),
    }
}

#[tokio::test]
async fn test_public_client_retry_on_server_error() {
    let mock_server = MockServer::start().await;

    // Fail first time with 500, succeed second time
    Mock::given(method("GET"))
        .and(path("/public/test"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let response_body = ApiResponse {
        status: 0,
        messages: None,
        data: "retry_success".to_string(),
        responsetime: None,
    };

    Mock::given(method("GET"))
        .and(path("/public/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let retry_config = RetryConfig {
        max_retries: 2,
        base_delay_ms: 1,
        max_delay_ms: 10,
    };

    let client = PublicRestClient::new(
        Some(retry_config),
        None,
        None,
        None,
        false,
        Some(mock_server.uri()),
    );

    let res: Result<String, GmoFxError> = client.get("/test").await;
    assert_eq!(res.unwrap(), "retry_success");
}

#[tokio::test]
async fn test_private_client_get_success_and_headers() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 0,
        messages: None,
        data: "private_get_success".to_string(),
        responsetime: None,
    };

    Mock::given(method("GET"))
        .and(path("/private/test"))
        .and(header("API-KEY", "test_api_key"))
        .and(header_exists("API-TIMESTAMP"))
        .and(header_exists("API-SIGN"))
        .and(|req: &wiremock::Request| {
            let api_timestamp = req.headers.get("API-TIMESTAMP").unwrap().to_str().unwrap();
            let api_sign = req.headers.get("API-SIGN").unwrap().to_str().unwrap();
            let signer = AuthSigner::new("test_api_key", "test_secret_key");
            let expected_sign = signer.sign(api_timestamp, "GET", "/test", "").api_sign;
            api_sign == expected_sign
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let auth = AuthSigner::new("test_api_key", "test_secret_key");
    let client =
        PrivateRestClient::new(auth, None, None, None, None, false, Some(mock_server.uri()));

    let res: Result<String, GmoFxError> = client.get("/test", None).await;
    assert_eq!(res.unwrap(), "private_get_success");
}

#[tokio::test]
async fn test_private_client_post_success() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 0,
        messages: None,
        data: "private_post_success".to_string(),
        responsetime: None,
    };

    #[derive(serde::Serialize)]
    struct RequestBody {
        foo: String,
    }

    Mock::given(method("POST"))
        .and(path("/private/test"))
        .and(body_json(serde_json::json!({ "foo": "bar" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let auth = AuthSigner::new("test_api_key", "test_secret_key");
    let client =
        PrivateRestClient::new(auth, None, None, None, None, false, Some(mock_server.uri()));

    let body = RequestBody {
        foo: "bar".to_string(),
    };
    let res: Result<String, GmoFxError> = client.post("/test", &body).await;
    assert_eq!(res.unwrap(), "private_post_success");
}

#[tokio::test]
async fn test_private_client_put_success() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 0,
        messages: None,
        data: "private_put_success".to_string(),
        responsetime: None,
    };

    #[derive(serde::Serialize)]
    struct RequestBody {
        foo: String,
    }

    Mock::given(method("PUT"))
        .and(path("/private/test"))
        .and(body_json(serde_json::json!({ "foo": "bar" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let auth = AuthSigner::new("test_api_key", "test_secret_key");
    let client =
        PrivateRestClient::new(auth, None, None, None, None, false, Some(mock_server.uri()));

    let body = RequestBody {
        foo: "bar".to_string(),
    };
    let res: Result<String, GmoFxError> = client.put("/test", &body).await;
    assert_eq!(res.unwrap(), "private_put_success");
}

#[tokio::test]
async fn test_private_client_delete_success() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 0,
        messages: None,
        data: "private_delete_success".to_string(),
        responsetime: None,
    };

    Mock::given(method("DELETE"))
        .and(path("/private/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let auth = AuthSigner::new("test_api_key", "test_secret_key");
    let client =
        PrivateRestClient::new(auth, None, None, None, None, false, Some(mock_server.uri()));

    let res: Result<String, GmoFxError> = client.delete("/test", None).await;
    assert_eq!(res.unwrap(), "private_delete_success");
}

#[tokio::test]
async fn test_private_client_api_error() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 10003,
        messages: Some(vec![ApiMessage {
            message_code: Some("10003".to_string()),
            message_string: Some("Authentication failed".to_string()),
        }]),
        data: "".to_string(),
        responsetime: None,
    };

    Mock::given(method("GET"))
        .and(path("/private/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let auth = AuthSigner::new("test_api_key", "test_secret_key");
    let client =
        PrivateRestClient::new(auth, None, None, None, None, false, Some(mock_server.uri()));

    let res: Result<String, GmoFxError> = client.get("/test", None).await;
    match res {
        Err(GmoFxError::Api { status, messages }) => {
            assert_eq!(status, 10003);
            let msgs = messages.unwrap();
            assert_eq!(
                msgs[0].message_string.as_deref().unwrap(),
                "Authentication failed"
            );
        }
        _ => panic!("Expected Api Error"),
    }
}
