/// Implied volatility solver.
///
/// Computes Black implied volatility to machine precision using Newton
/// iterations for the initial guess followed by Halley refinement.
///
/// The algorithm works in "normalised" space:
/// - Normalised price: beta = option_price / forward
/// - Log-moneyness: x = ln(F/K)
/// - Normalised volatility: s = sigma * sqrt(T)
use crate::math::constants::{DBL_EPSILON, DBL_MIN};
use crate::math::normal::{norm_cdf, norm_pdf};
use crate::pricing::error::PricingError;

// ── Normalised Black functions ───────────────────────────────────────────────

/// Compute the normalised Black call price.
///
/// Given log-moneyness `x = ln(F/K)` and normalised volatility `s = sigma*sqrt(T)`,
/// returns `beta_c = C / F = Phi(d1) - exp(-x) * Phi(d2)`
/// where `d1 = x/s + s/2`, `d2 = d1 - s`.
///
/// # Examples
/// ```
/// # use essvi::pricing::lets_be_rational::normalised_black_call;
/// let beta = normalised_black_call(0.0, 0.20);
/// assert!(beta > 0.0);
/// ```
pub fn normalised_black_call(x: f64, s: f64) -> f64 {
    if s <= 0.0 {
        return normalised_intrinsic_call(x);
    }
    let d1 = x / s + 0.5 * s;
    let d2 = d1 - s;
    norm_cdf(d1) - (-x).exp() * norm_cdf(d2)
}

/// Normalised intrinsic value for a call: max(1 - exp(-x), 0).
#[inline]
fn normalised_intrinsic_call(x: f64) -> f64 {
    if x <= 0.0 {
        0.0
    } else if x < 1e-6 {
        x * (1.0 - x * (0.5 - x / 6.0))
    } else {
        1.0 - (-x).exp()
    }
}

/// Compute the normalised Black vega: phi(d1) where d1 = x/s + s/2.
///
/// # Examples
/// ```
/// # use essvi::pricing::lets_be_rational::normalised_vega;
/// let v = normalised_vega(0.0, 0.20);
/// assert!(v > 0.0);
/// ```
pub fn normalised_vega(x: f64, s: f64) -> f64 {
    if s <= 0.0 {
        return 0.0;
    }
    let d1 = x / s + 0.5 * s;
    norm_pdf(d1)
}

// ── Implied volatility ───────────────────────────────────────────────────────

/// Compute Black implied volatility from an option price.
///
/// # Arguments
/// * `price` - Market option price (undiscounted)
/// * `forward` - Forward price
/// * `strike` - Strike price
/// * `t` - Time to expiry in years
/// * `q` - +1 for call, -1 for put
///
/// # Examples
/// ```
/// # use essvi::pricing::lets_be_rational::implied_volatility;
/// let price = essvi::pricing::black76::price(100.0, 100.0, 0.20, 1.0, 1).unwrap();
/// let iv = implied_volatility(price, 100.0, 100.0, 1.0, 1).unwrap();
/// assert!((iv - 0.20).abs() < 1e-12);
/// ```
pub fn implied_volatility(
    price: f64,
    forward: f64,
    strike: f64,
    t: f64,
    q: i32,
) -> Result<f64, PricingError> {
    if forward <= 0.0 {
        return Err(PricingError::InvalidInput(
            "forward must be positive".into(),
        ));
    }
    if strike <= 0.0 {
        return Err(PricingError::InvalidInput("strike must be positive".into()));
    }
    if t <= 0.0 {
        return Err(PricingError::InvalidInput(
            "time to expiry must be positive".into(),
        ));
    }
    if q != 1 && q != -1 {
        return Err(PricingError::InvalidInput(
            "q must be +1 (call) or -1 (put)".into(),
        ));
    }

    let intrinsic = if q == 1 {
        (forward - strike).max(0.0)
    } else {
        (strike - forward).max(0.0)
    };

    if price < intrinsic - DBL_EPSILON * forward {
        return Err(PricingError::BelowIntrinsic { price, intrinsic });
    }

    let max_price = if q == 1 { forward } else { strike };
    if price > max_price + DBL_EPSILON * forward {
        return Err(PricingError::AboveMaximum {
            price,
            maximum: max_price,
        });
    }

    // Convert to normalised call price via put-call parity if needed
    let call_price = if q == 1 {
        price
    } else {
        price + forward - strike
    };
    let beta_call = call_price / forward;
    let x = (forward / strike).ln();

    let s = compute_normalised_iv(beta_call, x);

    if s < 0.0 || !s.is_finite() {
        return Err(PricingError::InvalidInput(
            "implied volatility computation failed".into(),
        ));
    }

    Ok(s / t.sqrt())
}

/// Compute normalised implied volatility from a normalised option price.
///
/// # Arguments
/// * `beta` - Normalised option price
/// * `x` - Log-moneyness ln(F/K)
/// * `q` - +1 for call, -1 for put
///
/// # Examples
/// ```
/// # use essvi::pricing::lets_be_rational::{normalised_implied_volatility, normalised_black_call};
/// let s_true = 0.30;
/// let beta = normalised_black_call(0.0, s_true);
/// let s_recovered = normalised_implied_volatility(beta, 0.0, 1);
/// assert!((s_recovered - s_true).abs() < 1e-12);
/// ```
pub fn normalised_implied_volatility(beta: f64, x: f64, q: i32) -> f64 {
    let beta_call = if q == 1 {
        beta
    } else {
        beta + normalised_intrinsic_call(x)
    };
    compute_normalised_iv(beta_call, x)
}

/// Core: compute normalised IV via Newton + Halley.
fn compute_normalised_iv(beta_call: f64, x: f64) -> f64 {
    if beta_call <= 0.0 {
        return 0.0;
    }

    // Initial guess: Newton iterations from Brenner-Subrahmanyam starting point
    let s = newton_initial_guess(beta_call, x);

    if s <= 0.0 {
        return 0.0;
    }

    // Halley refinement (2 iterations)
    halley_refine(beta_call, x, s)
}

/// Find initial guess using bisection followed by Newton refinement.
fn newton_initial_guess(beta_call: f64, x: f64) -> f64 {
    if beta_call < DBL_MIN {
        return 0.0;
    }

    // Step 1: Find a bracket [s_lo, s_hi] via bisection
    // normalised_black_call is monotonically increasing in s
    let mut s_lo = DBL_EPSILON;
    let mut s_hi = 10.0;

    // Make sure s_hi brackets the solution
    while normalised_black_call(x, s_hi) < beta_call && s_hi < 1e6 {
        s_hi *= 2.0;
    }

    // Bisection to get close (10 iterations gives ~1e-3 relative accuracy)
    for _ in 0..30 {
        let s_mid = 0.5 * (s_lo + s_hi);
        let bc = normalised_black_call(x, s_mid);
        if bc < beta_call {
            s_lo = s_mid;
        } else {
            s_hi = s_mid;
        }
        if (s_hi - s_lo) < DBL_EPSILON * s_mid {
            break;
        }
    }

    let mut s = 0.5 * (s_lo + s_hi);

    // Step 2: Newton refinement from the bisection result
    for _ in 0..5 {
        let bc = normalised_black_call(x, s);
        let v = normalised_vega(x, s);

        if v < DBL_MIN {
            break;
        }

        let ds = (bc - beta_call) / v;
        s -= ds;

        if s <= 0.0 {
            s = 0.5 * (s_lo + s_hi);
            break;
        }

        if ds.abs() < DBL_EPSILON * s {
            break;
        }
    }

    s.max(DBL_EPSILON)
}

/// Halley refinement (3rd order, 2 iterations).
///
/// f(s) = normalised_black_call(x, s) - beta_target
/// f'(s) = phi(d1) = normalised_vega
/// f''(s) = phi'(d1) * dd1/ds = -d1 * phi(d1) * dd1/ds
///
/// dd1/ds = -x/s^2 + 1/2
///
/// Halley update: s -= f/f' / (1 - 0.5 * f/f' * f''/f')
fn halley_refine(beta_call: f64, x: f64, mut s: f64) -> f64 {
    for _ in 0..2 {
        let bc = normalised_black_call(x, s);
        let d1 = x / s + 0.5 * s;
        let vega = norm_pdf(d1);

        if vega < DBL_MIN {
            break;
        }

        let f = bc - beta_call;
        let u = f / vega; // Newton step

        // dd1/ds = -x/s^2 + 0.5
        let dd1_ds = -x / (s * s) + 0.5;

        // f''/f' = -d1 * dd1/ds
        let ratio = -d1 * dd1_ds;

        // Halley correction
        let denom = 1.0 - 0.5 * u * ratio;
        if denom.abs() < DBL_MIN {
            s -= u; // Fall back to Newton
        } else {
            s -= u / denom;
        }

        if s <= 0.0 {
            s = DBL_EPSILON;
        }
    }

    s.max(0.0)
}
