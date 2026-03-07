/// Pricing primitives for futures options.
///
/// - `error`: Typed error variants for invalid inputs and out-of-bounds prices
/// - `black76`: Undiscounted/discounted pricing, delta, gamma, vega, theta, and combined greeks
/// - `lets_be_rational`: Implied volatility solver (Let's Be Rational algorithm)
/// - `rational_cubic`: Rational cubic interpolation for initial guess refinement
pub mod black76;
pub mod error;
pub mod lets_be_rational;
pub mod rational_cubic;
