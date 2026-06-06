use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// GMO コイン FX で提供されている通貨ペアを表す列挙型です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FxSymbol {
    #[serde(rename = "USD_JPY")]
    UsdJpy,
    #[serde(rename = "EUR_JPY")]
    EurJpy,
    #[serde(rename = "GBP_JPY")]
    GbpJpy,
    #[serde(rename = "AUD_JPY")]
    AudJpy,
    #[serde(rename = "NZD_JPY")]
    NzdJpy,
    #[serde(rename = "CAD_JPY")]
    CadJpy,
    #[serde(rename = "CHF_JPY")]
    ChfJpy,
    #[serde(rename = "ZAR_JPY")]
    ZarJpy,
    #[serde(rename = "MXN_JPY")]
    MxnJpy,
    #[serde(rename = "TRY_JPY")]
    TryJpy,
    #[serde(rename = "EUR_USD")]
    EurUsd,
    #[serde(rename = "GBP_USD")]
    GbpUsd,
    #[serde(rename = "AUD_USD")]
    AudUsd,
    #[serde(rename = "NZD_USD")]
    NzdUsd,
    #[serde(rename = "EUR_GBP")]
    EurGbp,
}

impl FxSymbol {
    /// 通貨ペアの文字列表現を取得します。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UsdJpy => "USD_JPY",
            Self::EurJpy => "EUR_JPY",
            Self::GbpJpy => "GBP_JPY",
            Self::AudJpy => "AUD_JPY",
            Self::NzdJpy => "NZD_JPY",
            Self::CadJpy => "CAD_JPY",
            Self::ChfJpy => "CHF_JPY",
            Self::ZarJpy => "ZAR_JPY",
            Self::MxnJpy => "MXN_JPY",
            Self::TryJpy => "TRY_JPY",
            Self::EurUsd => "EUR_USD",
            Self::GbpUsd => "GBP_USD",
            Self::AudUsd => "AUD_USD",
            Self::NzdUsd => "NZD_USD",
            Self::EurGbp => "EUR_GBP",
        }
    }
}

impl fmt::Display for FxSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for FxSymbol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD_JPY" | "USDJPY" => Ok(Self::UsdJpy),
            "EUR_JPY" | "EURJPY" => Ok(Self::EurJpy),
            "GBP_JPY" | "GBPJPY" => Ok(Self::GbpJpy),
            "AUD_JPY" | "AUDJPY" => Ok(Self::AudJpy),
            "NZD_JPY" | "NZDJPY" => Ok(Self::NzdJpy),
            "CAD_JPY" | "CADJPY" => Ok(Self::CadJpy),
            "CHF_JPY" | "CHFJPY" => Ok(Self::ChfJpy),
            "ZAR_JPY" | "ZARJPY" => Ok(Self::ZarJpy),
            "MXN_JPY" | "MXNJPY" => Ok(Self::MxnJpy),
            "TRY_JPY" | "TRYJPY" => Ok(Self::TryJpy),
            "EUR_USD" | "EURUSD" => Ok(Self::EurUsd),
            "GBP_USD" | "GBPUSD" => Ok(Self::GbpUsd),
            "AUD_USD" | "AUDUSD" => Ok(Self::AudUsd),
            "NZD_USD" | "NZDUSD" => Ok(Self::NzdUsd),
            "EUR_GBP" | "EURGBP" => Ok(Self::EurGbp),
            _ => Err(format!("unknown symbol: {}", s)),
        }
    }
}

impl TryFrom<&str> for FxSymbol {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for FxSymbol {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_serialization() {
        let sym = FxSymbol::UsdJpy;
        let serialized = serde_json::to_string(&sym).unwrap();
        assert_eq!(serialized, "\"USD_JPY\"");

        let sym = FxSymbol::EurUsd;
        let serialized = serde_json::to_string(&sym).unwrap();
        assert_eq!(serialized, "\"EUR_USD\"");
    }

    #[test]
    fn test_serde_deserialization() {
        let serialized = "\"USD_JPY\"";
        let deserialized: FxSymbol = serde_json::from_str(serialized).unwrap();
        assert_eq!(deserialized, FxSymbol::UsdJpy);

        let serialized = "\"EUR_USD\"";
        let deserialized: FxSymbol = serde_json::from_str(serialized).unwrap();
        assert_eq!(deserialized, FxSymbol::EurUsd);

        let invalid = "\"INVALID_SYMBOL\"";
        let result: Result<FxSymbol, _> = serde_json::from_str(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(FxSymbol::from_str("USD_JPY").unwrap(), FxSymbol::UsdJpy);
        assert_eq!(FxSymbol::from_str("usdjpy").unwrap(), FxSymbol::UsdJpy);
        assert_eq!(FxSymbol::from_str("usd_jpy").unwrap(), FxSymbol::UsdJpy);
        assert_eq!(FxSymbol::from_str("USDJPY").unwrap(), FxSymbol::UsdJpy);
        assert_eq!(FxSymbol::from_str("eur_usd").unwrap(), FxSymbol::EurUsd);
        assert_eq!(FxSymbol::from_str("EURUSD").unwrap(), FxSymbol::EurUsd);

        assert!(FxSymbol::from_str("INVALID").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(FxSymbol::UsdJpy.to_string(), "USD_JPY");
        assert_eq!(FxSymbol::EurUsd.to_string(), "EUR_USD");
    }

    #[test]
    fn test_try_from() {
        let sym: FxSymbol = "USD_JPY".try_into().unwrap();
        assert_eq!(sym, FxSymbol::UsdJpy);

        let sym: FxSymbol = "usd_jpy".to_string().try_into().unwrap();
        assert_eq!(sym, FxSymbol::UsdJpy);

        let err: Result<FxSymbol, _> = "INVALID".try_into();
        assert!(err.is_err());
    }
}
