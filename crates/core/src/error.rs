use thiserror::Error;

pub type Result<T> = std::result::Result<T, GmoFxError>;

#[derive(Debug, Error)]
pub enum GmoFxError {
    #[error("http error: {0}")]
    Http(String),

    #[error("json error: {0}")]
    Json(String),

    #[error("api error: status={status}, messages={messages:?}")]
    Api {
        status: i64,
        messages: Option<Vec<ApiMessage>>,
    },

    #[error("missing api key or secret")]
    MissingCredentials,

    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiMessage {
    #[serde(default, rename = "message_code")]
    pub message_code: Option<String>,

    #[serde(default, rename = "message_string")]
    pub message_string: Option<String>,
}
