use std::collections::HashMap;
use gmo_coin_fx_core::{models::Ticker, GmoFxError, Result};

/// Spread threshold configuration for a specific symbol.
#[derive(Debug, Clone)]
pub struct SpreadThreshold {
    pub symbol: String,
    pub max_spread: String,
}

/// Evaluation result of the current spread.
#[derive(Debug, Clone)]
pub struct SpreadEvaluation {
    pub symbol: String,
    pub bid: String,
    pub ask: String,
    pub spread: String,
    pub max_spread: String,
    pub acceptable: bool,
}

/// Trait defining the spread width guard.
pub trait SpreadGuard {
    fn evaluate(&self, ticker: &Ticker) -> Result<SpreadEvaluation>;
}

/// A static spread guard implementation that uses a pre-configured map of thresholds.
pub struct StaticSpreadGuard {
    thresholds: HashMap<String, SpreadThreshold>,
}

impl StaticSpreadGuard {
    /// Creates a new `StaticSpreadGuard` with the given list of thresholds.
    pub fn new(thresholds: Vec<SpreadThreshold>) -> Self {
        let mut map = HashMap::new();
        for t in thresholds {
            map.insert(t.symbol.clone(), t);
        }
        Self { thresholds: map }
    }
}

impl SpreadGuard for StaticSpreadGuard {
    fn evaluate(&self, ticker: &Ticker) -> Result<SpreadEvaluation> {
        let bid_val = parse_fixed(&ticker.bid).ok_or_else(|| {
            GmoFxError::InvalidRequest(format!("Invalid bid price: {}", ticker.bid))
        })?;
        let ask_val = parse_fixed(&ticker.ask).ok_or_else(|| {
            GmoFxError::InvalidRequest(format!("Invalid ask price: {}", ticker.ask))
        })?;

        if ask_val < bid_val {
            return Err(GmoFxError::InvalidRequest(format!(
                "Ask price {} is less than bid price {}",
                ticker.ask, ticker.bid
            )));
        }

        let spread_val = ask_val - bid_val;
        let spread_str = format_fixed(spread_val);

        if let Some(threshold) = self.thresholds.get(&ticker.symbol) {
            let max_spread_val = parse_fixed(&threshold.max_spread).ok_or_else(|| {
                GmoFxError::InvalidRequest(format!(
                    "Invalid max spread threshold: {}",
                    threshold.max_spread
                ))
            })?;

            let acceptable = spread_val <= max_spread_val;

            Ok(SpreadEvaluation {
                symbol: ticker.symbol.clone(),
                bid: ticker.bid.clone(),
                ask: ticker.ask.clone(),
                spread: spread_str,
                max_spread: threshold.max_spread.clone(),
                acceptable,
            })
        } else {
            // Missing threshold => acceptable = false
            Ok(SpreadEvaluation {
                symbol: ticker.symbol.clone(),
                bid: ticker.bid.clone(),
                ask: ticker.ask.clone(),
                spread: spread_str,
                max_spread: "".to_string(),
                acceptable: false,
            })
        }
    }
}

/// Parses a decimal string into a fixed-point `i64` scaled by 1,000,000.
fn parse_fixed(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Check sign
    let (is_negative, s) = if let Some(stripped) = s.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, s)
    };

    let mut parts = s.split('.');
    let integer_part = parts.next()?;
    let fractional_part = parts.next();
    if parts.next().is_some() {
        return None;
    }

    // Parse integer part
    let mut val = integer_part.parse::<i64>().ok()?;
    val = val.checked_mul(1_000_000)?;

    // Parse fractional part (take up to 6 digits, pad with 0s if less)
    if let Some(frac) = fractional_part {
        let mut frac_val = 0;
        let mut multiplier = 100_000;
        for (i, c) in frac.chars().enumerate() {
            if i >= 6 {
                break;
            }
            let digit = c.to_digit(10)? as i64;
            frac_val += digit * multiplier;
            multiplier /= 10;
        }
        val = val.checked_add(frac_val)?;
    }

    if is_negative {
        val = -val;
    }
    Some(val)
}

/// Formats a fixed-point `i64` scaled by 1,000,000 back to a decimal string.
/// Trims trailing zeros, keeping at least 3 decimal places (e.g. "0.010").
fn format_fixed(val: i64) -> String {
    let is_negative = val < 0;
    let val = val.abs();
    let integer = val / 1_000_000;
    let fractional = val % 1_000_000;
    let sign = if is_negative { "-" } else { "" };

    let mut frac_str = format!("{:06}", fractional);
    while frac_str.ends_with('0') && frac_str.len() > 3 {
        frac_str.pop();
    }

    format!("{}{}.{}", sign, integer, frac_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptable_spread() {
        let guard = StaticSpreadGuard::new(vec![SpreadThreshold {
            symbol: "USD_JPY".to_string(),
            max_spread: "0.020".to_string(),
        }]);

        let ticker = Ticker {
            symbol: "USD_JPY".to_string(),
            bid: "158.940".to_string(),
            ask: "158.950".to_string(),
            timestamp: "".to_string(),
            status: "".to_string(),
        };

        let result = guard.evaluate(&ticker).unwrap();
        assert_eq!(result.symbol, "USD_JPY");
        assert_eq!(result.bid, "158.940");
        assert_eq!(result.ask, "158.950");
        assert_eq!(result.spread, "0.010");
        assert_eq!(result.max_spread, "0.020");
        assert!(result.acceptable);
    }

    #[test]
    fn test_too_wide_spread() {
        let guard = StaticSpreadGuard::new(vec![SpreadThreshold {
            symbol: "USD_JPY".to_string(),
            max_spread: "0.020".to_string(),
        }]);

        let ticker = Ticker {
            symbol: "USD_JPY".to_string(),
            bid: "158.940".to_string(),
            ask: "158.980".to_string(),
            timestamp: "".to_string(),
            status: "".to_string(),
        };

        let result = guard.evaluate(&ticker).unwrap();
        assert_eq!(result.symbol, "USD_JPY");
        assert_eq!(result.bid, "158.940");
        assert_eq!(result.ask, "158.980");
        assert_eq!(result.spread, "0.040");
        assert_eq!(result.max_spread, "0.020");
        assert!(!result.acceptable);
    }

    #[test]
    fn test_missing_threshold() {
        let guard = StaticSpreadGuard::new(vec![]);

        let ticker = Ticker {
            symbol: "USD_JPY".to_string(),
            bid: "158.940".to_string(),
            ask: "158.950".to_string(),
            timestamp: "".to_string(),
            status: "".to_string(),
        };

        let result = guard.evaluate(&ticker).unwrap();
        assert_eq!(result.symbol, "USD_JPY");
        assert_eq!(result.spread, "0.010");
        assert_eq!(result.max_spread, "");
        assert!(!result.acceptable); // Missing threshold => acceptable = false
    }

    #[test]
    fn test_invalid_bid_price() {
        let guard = StaticSpreadGuard::new(vec![SpreadThreshold {
            symbol: "USD_JPY".to_string(),
            max_spread: "0.020".to_string(),
        }]);

        let ticker = Ticker {
            symbol: "USD_JPY".to_string(),
            bid: "invalid".to_string(),
            ask: "158.950".to_string(),
            timestamp: "".to_string(),
            status: "".to_string(),
        };

        let result = guard.evaluate(&ticker);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ask_price() {
        let guard = StaticSpreadGuard::new(vec![SpreadThreshold {
            symbol: "USD_JPY".to_string(),
            max_spread: "0.020".to_string(),
        }]);

        let ticker = Ticker {
            symbol: "USD_JPY".to_string(),
            bid: "158.940".to_string(),
            ask: "invalid".to_string(),
            timestamp: "".to_string(),
            status: "".to_string(),
        };

        let result = guard.evaluate(&ticker);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_max_spread() {
        let guard = StaticSpreadGuard::new(vec![SpreadThreshold {
            symbol: "USD_JPY".to_string(),
            max_spread: "invalid".to_string(),
        }]);

        let ticker = Ticker {
            symbol: "USD_JPY".to_string(),
            bid: "158.940".to_string(),
            ask: "158.950".to_string(),
            timestamp: "".to_string(),
            status: "".to_string(),
        };

        let result = guard.evaluate(&ticker);
        assert!(result.is_err());
    }

    #[test]
    fn test_fixed_point_parser() {
        assert_eq!(parse_fixed("0.02"), Some(20_000));
        assert_eq!(parse_fixed("0.020"), Some(20_000));
        assert_eq!(parse_fixed("1.08234"), Some(1_082_340));
        assert_eq!(parse_fixed("158.940"), Some(158_940_000));
        assert_eq!(parse_fixed("-158.940"), Some(-158_940_000));
        assert_eq!(parse_fixed("158"), Some(158_000_000));
        assert_eq!(parse_fixed(""), None);
        assert_eq!(parse_fixed("abc"), None);
        assert_eq!(parse_fixed("1.2.3"), None);
    }
}
