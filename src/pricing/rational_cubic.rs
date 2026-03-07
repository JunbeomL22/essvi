/// Rational cubic interpolation for implied volatility initial guesses.
///
/// Implements the rational cubic interpolation from Peter Jaeckel's
/// "Let's Be Rational" paper, used to produce accurate initial guesses
/// for the Householder iteration.

/// Rational cubic interpolation.
///
/// Given endpoints (x_l, y_l) with derivative d_l and (x_r, y_r) with
/// derivative d_r, compute the interpolated value at x.
///
/// The rational cubic is a ratio of cubics that passes through both
/// endpoints with the specified derivatives, providing shape control
/// via the `prefer_shape_preservation_over_smoothness` parameter.
pub fn rational_cubic_interpolation(
    x: f64,
    x_l: f64,
    x_r: f64,
    y_l: f64,
    y_r: f64,
    d_l: f64,
    d_r: f64,
    prefer_shape_preservation_over_smoothness: bool,
) -> f64 {
    let h = x_r - x_l;
    if h.abs() <= 0.0 {
        return 0.5 * (y_l + y_r);
    }

    let t = (x - x_l) / h;
    let omt = 1.0 - t;

    if t <= 0.0 {
        return y_l;
    }
    if t >= 1.0 {
        return y_r;
    }

    // Delta and slopes
    let delta = (y_r - y_l) / h;
    let d_l_scaled = d_l * h;
    let d_r_scaled = d_r * h;

    // Compute the rational cubic with shape control
    let r = compute_r(
        delta,
        d_l_scaled,
        d_r_scaled,
        prefer_shape_preservation_over_smoothness,
    );

    // Numerator: interpolating cubic
    let num = t * (y_r * (1.0 + r * omt) + d_r_scaled * omt)
        + omt * (y_l * (1.0 + r * t) - d_l_scaled * t);

    // Denominator
    let den = 1.0 + r * t * omt;

    num / den
}

/// Compute the shape control parameter r.
///
/// r controls the shape of the rational cubic. Larger r values pull
/// the curve toward the linear interpolant, smaller values allow
/// more curvature. Negative r can cause poles.
fn compute_r(delta: f64, d_l_scaled: f64, d_r_scaled: f64, prefer_shape: bool) -> f64 {
    // From Jaeckel: r is chosen to ensure monotonicity when possible
    let s_l = d_l_scaled - delta;
    let s_r = d_r_scaled - delta;

    // If the slopes have the same sign as delta (monotone data),
    // ensure the interpolant is also monotone
    if prefer_shape {
        // Minimum r to ensure no poles and shape preservation
        let r = if s_l * s_r > 0.0 {
            // Non-monotone: use convexity-preserving r
            let r_candidate = (s_l * s_l + s_r * s_r) / (s_l * s_r);
            r_candidate.max(1.5)
        } else {
            // Monotone or flat: minimal shape control
            let discriminant = s_l * s_l + s_r * s_r - s_l * s_r;
            if discriminant >= 0.0 {
                // Enough curvature is available
                (discriminant.sqrt() + 0.5 * (s_l + s_r).abs()) / (s_l - s_r).abs().max(1e-100)
            } else {
                0.0
            }
        };
        r.max(0.0)
    } else {
        // Smoothness-preserving: use the natural r
        let sum = s_l + s_r;
        if sum.abs() < 1e-100 {
            0.0
        } else {
            let r = (s_l * s_l + s_r * s_r) / (sum * sum) * 3.0;
            r.max(0.0)
        }
    }
}
