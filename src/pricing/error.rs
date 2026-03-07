/// Error types for Black-76 pricing operations.
///
/// Provides typed error variants instead of panics for invalid inputs
/// and out-of-bounds option prices.
use std::fmt;

/// Error type for pricing operations.
///
/// Returned when inputs are invalid or option prices fall outside
/// the feasible range for the Black-76 model.
///
/// # Examples
/// ```
/// # use essvi::pricing::error::PricingError;
/// let err = PricingError::InvalidInput("sigma must be positive".into());
/// assert!(matches!(err, PricingError::InvalidInput(_)));
/// ```
#[derive(Debug, Clone)]
pub enum PricingError {
    /// Option price exceeds the theoretical maximum.
    ///
    /// For calls: price > forward (undiscounted).
    /// For puts: price > strike (undiscounted).
    AboveMaximum { price: f64, maximum: f64 },

    /// Option price is below intrinsic value.
    ///
    /// For calls: price < max(forward - strike, 0).
    /// For puts: price < max(strike - forward, 0).
    BelowIntrinsic { price: f64, intrinsic: f64 },

    /// Input parameters are invalid (negative sigma, non-positive time, etc.).
    InvalidInput(String),
}

impl fmt::Display for PricingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PricingError::AboveMaximum { price, maximum } => {
                write!(
                    f,
                    "option price {:.6e} exceeds theoretical maximum {:.6e}",
                    price, maximum
                )
            }
            PricingError::BelowIntrinsic { price, intrinsic } => {
                write!(
                    f,
                    "option price {:.6e} is below intrinsic value {:.6e}",
                    price, intrinsic
                )
            }
            PricingError::InvalidInput(msg) => {
                write!(f, "invalid input: {}", msg)
            }
        }
    }
}

impl std::error::Error for PricingError {}
