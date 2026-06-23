use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: i64,
    pub data: T,

    #[serde(default)]
    pub messages: Option<Vec<crate::ApiMessage>>,

    #[serde(default)]
    pub responsetime: Option<String>,
}
