use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// HMAC-SHA256 署名を用いて GMO コイン FX API の認証ヘッダーを生成します。
#[derive(Debug, Clone)]
pub struct AuthSigner {
    api_key: String,
    secret_key: String,
}

/// API リクエストに付与する認証ヘッダーの値。
#[derive(Debug, Clone)]
pub struct AuthHeaders {
    /// `API-KEY` ヘッダー
    pub api_key: String,
    /// `API-TIMESTAMP` ヘッダー
    pub api_timestamp: String,
    /// `API-SIGN` ヘッダー（HMAC-SHA256 の hex エンコード）
    pub api_sign: String,
}

impl AuthSigner {
    /// 新しい [`AuthSigner`] を生成します。
    pub fn new(api_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            secret_key: secret_key.into(),
        }
    }

    /// タイムスタンプ・メソッド・パス・ボディから署名を計算し [`AuthHeaders`] を返します。
    pub fn sign(&self, timestamp: &str, method: &str, path: &str, body: &str) -> AuthHeaders {
        let text = format!("{timestamp}{method}{path}{body}");

        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC accepts keys of any size");

        mac.update(text.as_bytes());

        AuthHeaders {
            api_key: self.api_key.clone(),
            api_timestamp: timestamp.to_string(),
            api_sign: hex::encode(mac.finalize().into_bytes()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_is_hex_sha256_length() {
        let signer = AuthSigner::new("key", "secret");
        let headers = signer.sign("1700000000000", "GET", "/v1/account/assets", "");

        assert_eq!(headers.api_key, "key");
        assert_eq!(headers.api_sign.len(), 64);
    }
}
