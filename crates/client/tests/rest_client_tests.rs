use gmo_coin_fx_client::auth::AuthSigner;
use gmo_coin_fx_client::gateway::RetryConfig;
use gmo_coin_fx_client::{PrivateRestClient, PublicRestClient};
use gmo_coin_fx_core::{models::ApiResponse, GmoFxError};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_public_client_success() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 0,
        messages: vec![],
        data: "public_success".to_string(),
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
        messages: vec!["Invalid request".to_string()],
        data: "".to_string(),
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
            assert_eq!(messages[0], "Invalid request");
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
        messages: vec![],
        data: "retry_success".to_string(),
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
        messages: vec![],
        data: "private_get_success".to_string(),
    };

    Mock::given(method("GET"))
        .and(path("/private/test"))
        .and(header("API-KEY", "test_api_key"))
        .and(header("API-TIMESTAMP", wiremock::matchers::any()))
        .and(header("API-SIGN", wiremock::matchers::any()))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let auth = AuthSigner::new("test_api_key", "test_secret_key");
    let client = PrivateRestClient::new(
        auth,
        None,
        None,
        None,
        None,
        false,
        Some(mock_server.uri()),
    );

    let res: Result<String, GmoFxError> = client.get("/test", None).await;
    assert_eq!(res.unwrap(), "private_get_success");
}

#[tokio::test]
async fn test_private_client_post_success() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 0,
        messages: vec![],
        data: "private_post_success".to_string(),
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
    let client = PrivateRestClient::new(
        auth,
        None,
        None,
        None,
        None,
        false,
        Some(mock_server.uri()),
    );

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
        messages: vec![],
        data: "private_put_success".to_string(),
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
    let client = PrivateRestClient::new(
        auth,
        None,
        None,
        None,
        None,
        false,
        Some(mock_server.uri()),
    );

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
        messages: vec![],
        data: "private_delete_success".to_string(),
    };

    Mock::given(method("DELETE"))
        .and(path("/private/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let auth = AuthSigner::new("test_api_key", "test_secret_key");
    let client = PrivateRestClient::new(
        auth,
        None,
        None,
        None,
        None,
        false,
        Some(mock_server.uri()),
    );

    let res: Result<String, GmoFxError> = client.delete("/test", None).await;
    assert_eq!(res.unwrap(), "private_delete_success");
}

#[tokio::test]
async fn test_private_client_api_error() {
    let mock_server = MockServer::start().await;

    let response_body = ApiResponse {
        status: 10003,
        messages: vec!["Authentication failed".to_string()],
        data: "".to_string(),
    };

    Mock::given(method("GET"))
        .and(path("/private/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let auth = AuthSigner::new("test_api_key", "test_secret_key");
    let client = PrivateRestClient::new(
        auth,
        None,
        None,
        None,
        None,
        false,
        Some(mock_server.uri()),
    );

    let res: Result<String, GmoFxError> = client.get("/test", None).await;
    match res {
        Err(GmoFxError::Api { status, messages }) => {
            assert_eq!(status, 10003);
            assert_eq!(messages[0], "Authentication failed");
        }
        _ => panic!("Expected Api Error"),
    }
}
