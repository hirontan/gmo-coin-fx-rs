pub mod private_rest_client;
pub mod public_rest_client;

pub use private_rest_client::PrivateRestClient;
pub use public_rest_client::PublicRestClient;

use crate::auth::AuthSigner;
use gmo_coin_fx_core::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// パブリック／プライベート REST クライアントを束ねるファサード。
///
/// 上位の [`crate::gateway`] からはこの型を介して API を呼び出します。
#[derive(Clone)]
pub struct RestClient {
    pub(crate) public: PublicRestClient,
    pub(crate) private: Option<PrivateRestClient>,
}

impl RestClient {
    /// 新しい [`RestClient`] を生成します。
    ///
    /// `auth` が `None` の場合はパブリック API のみ利用可能です。
    pub fn new(
        auth: Option<AuthSigner>,
        retry_config: Option<crate::gateway::RetryConfig>,
        timeout: Option<std::time::Duration>,
        connect_timeout: Option<std::time::Duration>,
        base_url: Option<String>,
    ) -> Self {
        Self {
            public: PublicRestClient::new(retry_config, timeout, connect_timeout, base_url.clone()),
            private: auth
                .map(|a| PrivateRestClient::new(a, retry_config, timeout, connect_timeout, base_url)),
        }
    }

    // ─── Public ────────────────────────────────────────────────────────

    /// パブリック GET リクエストを送信します。
    pub async fn public_get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.public.get(path).await
    }

    // ─── Private ───────────────────────────────────────────────────────

    fn priv_client(&self) -> Result<&PrivateRestClient> {
        self.private
            .as_ref()
            .ok_or(gmo_coin_fx_core::GmoFxError::MissingCredentials)
    }

    /// プライベート GET リクエストを送信します。
    pub async fn private_get<T>(&self, path: &str, query: Option<&[(&str, String)]>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.priv_client()?.get(path, query).await
    }

    /// プライベート POST リクエストを送信します。
    pub async fn private_post<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.priv_client()?.post(path, body).await
    }

    /// プライベート PUT リクエストを送信します。
    pub async fn private_put<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.priv_client()?.put(path, body).await
    }

    /// プライベート DELETE リクエストを送信します。
    pub async fn private_delete<T>(&self, path: &str, query: Option<&[(&str, String)]>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.priv_client()?.delete(path, query).await
    }
}
