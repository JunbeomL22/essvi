/// Mathematical building blocks for Black-76 pricing and implied volatility.
///
/// - `constants`: Machine-precision thresholds, mathematical constants, algorithm boundaries
/// - `erf`: Cody's erf/erfc/erfcx with rational Chebyshev approximations
/// - `normal`: Standard-precision normal distribution (PDF, CDF, inverse CDF)
/// - `normal_hp`: High-precision normal distribution with asymptotic tail expansion
pub mod constants;
pub mod erf;
pub mod normal;
pub mod normal_hp;
