/// High-precision normal distribution functions.
///
/// Provides a normal CDF that uses asymptotic expansion for extreme tail
/// arguments, avoiding premature return of 0.0 or 1.0. Also provides a
/// high-precision inverse CDF based on rational approximations with
/// Halley refinement.
///
/// These are needed by the implied volatility solver where standard precision
/// CDF underflows in deep OTM/ITM regimes.
use crate::math::constants::{
    DBL_EPSILON, DBL_MIN, NORM_CDF_ASYMPTOTIC_EXPANSION_FIRST_THRESHOLD,
    NORM_CDF_ASYMPTOTIC_EXPANSION_SECOND_THRESHOLD, ONE_OVER_SQRT_TWO, ONE_OVER_SQRT_TWO_PI,
};
use crate::math::erf;

/// High-precision standard normal PDF.
///
/// Same as standard norm_pdf but included for API completeness in the
/// high-precision module.
#[inline]
pub fn norm_pdf(x: f64) -> f64 {
    ONE_OVER_SQRT_TWO_PI * (-0.5 * x * x).exp()
}

/// High-precision standard normal CDF with asymptotic expansion for tails.
///
/// For moderate arguments, uses `erfc` for accuracy.
/// For extreme negative arguments (x < -10), uses an asymptotic expansion
/// of the Mills ratio to avoid returning exactly 0.0.
///
/// The asymptotic expansion is:
///   Phi(x) ~ phi(x)/|x| * sum_{k=0}^{N} (-1)^k * (2k-1)!! / x^{2k}
///
/// # Examples
/// ```
/// # use essvi::math::normal_hp::norm_cdf;
/// assert!((norm_cdf(0.0) - 0.5).abs() < 1e-15);
/// // Does not return 0.0 for extreme arguments
/// assert!(norm_cdf(-37.0) > 0.0);
/// ```
pub fn norm_cdf(x: f64) -> f64 {
    if x > NORM_CDF_ASYMPTOTIC_EXPANSION_FIRST_THRESHOLD {
        // Standard region: use erfc for accuracy
        0.5 * erf::erfc(-x * ONE_OVER_SQRT_TWO)
    } else if x > NORM_CDF_ASYMPTOTIC_EXPANSION_SECOND_THRESHOLD {
        // First asymptotic region: series expansion
        norm_cdf_asymptotic_expansion(x)
    } else {
        // Extreme tail: return smallest representable positive value
        // rather than 0.0
        DBL_MIN
    }
}

/// Asymptotic expansion for the normal CDF in the far left tail.
///
/// Uses the identity:
///   Phi(x) = phi(|x|) / |x| * (1 - 1/x^2 + 1*3/x^4 - 1*3*5/x^6 + ...)
///
/// The series is asymptotic (divergent). We stop adding terms when they
/// begin growing or drop below machine epsilon relative to the running sum.
fn norm_cdf_asymptotic_expansion(x: f64) -> f64 {
    let ax = x.abs();
    let xsq = ax * ax;

    let mut sum: f64 = 1.0;
    let mut term: f64 = 1.0;

    for k in 1..100 {
        let prev_term_abs = term.abs();
        term *= -(2.0 * k as f64 - 1.0) / xsq;

        // Stop when terms start growing (divergence) or are negligible
        if term.abs() > prev_term_abs {
            break;
        }
        if term.abs() < DBL_EPSILON * sum.abs() {
            sum += term;
            break;
        }

        sum += term;
    }

    // Phi(x) for x < 0:  phi(|x|) / |x| * sum
    let result = ONE_OVER_SQRT_TWO_PI * (-0.5 * xsq).exp() / ax * sum;
    result.max(DBL_MIN)
}

/// High-precision inverse normal CDF.
///
/// Uses rational approximations with Halley refinement for machine-precision
/// accuracy, suitable for the Let's Be Rational implied volatility algorithm.
///
/// Covers the full domain including extreme tails where standard inverse CDF
/// implementations lose precision.
///
/// # Arguments
/// * `p` - Probability in (0, 1). Returns +/-inf at boundaries, NaN for invalid.
///
/// # Examples
/// ```
/// # use essvi::math::normal_hp::norm_inv_cdf;
/// assert!(norm_inv_cdf(0.5).abs() < 1e-15);
/// ```
pub fn norm_inv_cdf(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if p != p {
        return f64::NAN;
    }

    // Acklam's rational approximation coefficients
    const A: [f64; 6] = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];

    const B: [f64; 5] = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];

    const C: [f64; 6] = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];

    const D: [f64; 4] = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    let initial = if p < P_LOW {
        // Lower tail
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        // Central region
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        // Upper tail
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };

    // Halley refinement: uses Phi(x) and phi(x) to refine the initial guess.
    // Given initial approximation x0 for Phi^{-1}(p):
    //   e = Phi(x0) - p
    //   u = e * sqrt(2*pi) * exp(x0^2 / 2)
    //   x1 = x0 - u / (1 + x0 * u / 2)
    halley_refine(initial, p)
}

/// Halley refinement step for inverse CDF.
///
/// Given an initial approximation `x` for `Phi^{-1}(p)`, refine it
/// using one step of Halley's rational method.
fn halley_refine(x: f64, p: f64) -> f64 {
    let phi_x = norm_cdf(x);
    let pdf_x = norm_pdf(x);
    if pdf_x.abs() < DBL_MIN {
        return x;
    }
    let e = phi_x - p;
    let u = e / pdf_x;
    // Halley correction: x - u / (1 + x*u/2)
    x - u / (1.0 + 0.5 * x * u)
}
