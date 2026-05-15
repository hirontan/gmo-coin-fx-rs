use thiserror::Error;

pub type Result<T> = std::result::Result<T, RiskError>;

#[derive(Debug, Error, PartialEq)]
pub enum RiskError {
    #[error("invalid quantity: {0}")]
    InvalidQuantity(f64),

    #[error("invalid price: {0}")]
    InvalidPrice(f64),

    #[error("invalid leverage: {0}")]
    InvalidLeverage(f64),

    #[error("invalid equity: {0}")]
    InvalidEquity(f64),

    #[error("invalid margin: {0}")]
    InvalidMargin(f64),

    #[error("invalid risk percentage: {0}")]
    InvalidRiskPct(f64),

    #[error("invalid stop distance: {0}")]
    InvalidStopDistance(f64),
}
