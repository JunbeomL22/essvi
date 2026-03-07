/// Standard-precision normal distribution functions: PDF, CDF, and inverse CDF.
///
/// CDF uses the error function from `crate::math::erf`.
/// Inverse CDF uses Acklam's rational approximation with Halley refinement.
use crate::math::constants::{ONE_OVER_SQRT_TWO, ONE_OVER_SQRT_TWO_PI};
use crate::math::erf;

/// Standard normal probability density function.
///
/// phi(x) = (1/sqrt(2*pi)) * exp(-x^2/2)
///
/// # Examples
/// ```
/// # use essvi::math::normal::norm_pdf;
/// assert!((norm_pdf(0.0) - 0.3989422804014327).abs() < 1e-15);
/// ```
#[inline]
pub fn norm_pdf(x: f64) -> f64 {
    ONE_OVER_SQRT_TWO_PI * (-0.5 * x * x).exp()
}

/// Standard normal cumulative distribution function.
///
/// Phi(x) = erfc(-x / sqrt(2)) / 2
///
/// Uses Cody's erfc implementation for machine-precision accuracy.
///
/// # Examples
/// ```
/// # use essvi::math::normal::norm_cdf;
/// assert!((norm_cdf(0.0) - 0.5).abs() < 1e-15);
/// assert!((norm_cdf(1.0) - 0.8413447460685429).abs() < 1e-14);
/// ```
#[inline]
pub fn norm_cdf(x: f64) -> f64 {
    0.5 * erf::erfc(-x * ONE_OVER_SQRT_TWO)
}

/// Inverse of the standard normal CDF (quantile function / probit).
///
/// Uses Acklam's rational approximation with one step of Halley refinement.
/// Accurate to machine precision across the full domain.
///
/// Returns -inf for p=0, +inf for p=1, NaN for p outside [0,1].
///
/// # Examples
/// ```
/// # use essvi::math::normal::norm_inv_cdf;
/// assert!(norm_inv_cdf(0.5).abs() < 1e-15);
/// assert!((norm_inv_cdf(0.8413447460685429) - 1.0).abs() < 1e-8);
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

    // Halley refinement: given initial approximation x for Phi^{-1}(p):
    //   e = Phi(x) - p
    //   u = e / phi(x)
    //   x_new = x - u / (1 + x*u/2)
    let phi_x = norm_cdf(initial);
    let pdf_x = norm_pdf(initial);
    if pdf_x.abs() < f64::MIN_POSITIVE {
        return initial;
    }
    let e = phi_x - p;
    let u = e / pdf_x;
    initial - u / (1.0 + 0.5 * initial * u)
}
