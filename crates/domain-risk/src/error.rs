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
}
