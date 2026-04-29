use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct AuthSigner {
    api_key: String,
    secret_key: String,
}

#[derive(Debug, Clone)]
pub struct AuthHeaders {
    pub api_key: String,
    pub api_timestamp: String,
    pub api_sign: String,
}

impl AuthSigner {
    pub fn new(api_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            secret_key: secret_key.into(),
        }
    }

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
