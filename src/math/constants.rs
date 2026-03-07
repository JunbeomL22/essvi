/// Numerical constants used by Black-76 pricing and implied volatility algorithms.
///
/// Machine-precision thresholds, mathematical constants, and algorithm-specific
/// boundaries collected in one place for downstream modules.

// ── Machine precision ───────────────────────────────────────────────────────

/// Machine epsilon for f64 (2^-52).
pub const DBL_EPSILON: f64 = f64::EPSILON; // 2.220446049250313e-16

/// Square root of machine epsilon.
pub const SQRT_DBL_EPSILON: f64 = 1.4901161193847656e-8;

/// Fourth root of machine epsilon.
pub const FOURTH_ROOT_DBL_EPSILON: f64 = 1.2207031250000000e-4;

/// Eighth root of machine epsilon.
pub const EIGHTH_ROOT_DBL_EPSILON: f64 = 1.1048543456039805e-2;

/// Sixteenth root of machine epsilon.
pub const SIXTEENTH_ROOT_DBL_EPSILON: f64 = 1.0510814151985718e-1;

/// Smallest positive normal f64 (2^-1022).
pub const DBL_MIN: f64 = f64::MIN_POSITIVE; // 2.2250738585072014e-308

/// Largest finite f64.
pub const DBL_MAX: f64 = f64::MAX; // 1.7976931348623158e+308

/// Natural log of DBL_MIN.
pub const DBL_MIN_LN: f64 = -708.3964185322641;

/// Natural log of DBL_MAX.
pub const DBL_MAX_LN: f64 = 709.782712893384;

// ── Mathematical constants ──────────────────────────────────────────────────

/// sqrt(2*pi)
pub const SQRT_TWO_PI: f64 = 2.5066282746310002;

/// 1 / sqrt(2*pi)
pub const ONE_OVER_SQRT_TWO_PI: f64 = 0.3989422804014327;

/// sqrt(2)
pub const SQRT_TWO: f64 = std::f64::consts::SQRT_2;

/// 1 / sqrt(2)
pub const ONE_OVER_SQRT_TWO: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// pi
pub const PI: f64 = std::f64::consts::PI;

/// 2 / sqrt(pi) — used in erf/erfc
pub const TWO_OVER_SQRT_PI: f64 = std::f64::consts::FRAC_2_SQRT_PI;

// ── Algorithm thresholds ────────────────────────────────────────────────────

/// Codename threshold: |x| below this uses the Taylor series path in erf.
pub const ERF_SMALL_THRESHOLD: f64 = 0.5;

/// Codename threshold: |x| above this uses the large-argument erfc path.
pub const ERFC_LARGE_THRESHOLD: f64 = 4.0;

/// Threshold for switching to asymptotic expansion in high-precision normal CDF.
pub const NORM_CDF_ASYMPTOTIC_EXPANSION_FIRST_THRESHOLD: f64 = -10.0;

/// Second asymptotic threshold for extreme tails.
pub const NORM_CDF_ASYMPTOTIC_EXPANSION_SECOND_THRESHOLD: f64 = -1.0 / SQRT_DBL_EPSILON;

/// Denormalisation cutoff for normal PDF/CDF computations.
pub const DENORMALISATION_CUTOFF: f64 = 0.0;
