/// Black-76 pricing model for futures options.
///
/// Provides undiscounted and discounted option pricing, individual greeks
/// (delta, gamma, vega, theta), and a combined greeks computation.
///
/// Uses the call/put convention `q = +1` for calls, `q = -1` for puts,
/// following the Let's Be Rational convention.
use crate::math::normal::{norm_cdf, norm_pdf};
use crate::pricing::error::PricingError;

/// All greeks for a Black-76 option, returned by [`greeks`].
#[derive(Debug, Clone, Copy)]
pub struct Greeks {
    /// Undiscounted option price.
    pub price: f64,
    /// Delta: dPrice/dForward (undiscounted).
    pub delta: f64,
    /// Gamma: d2Price/dForward2 (undiscounted).
    pub gamma: f64,
    /// Vega: dPrice/dSigma (undiscounted).
    pub vega: f64,
    /// Theta: dPrice/dT (undiscounted, with respect to time-to-expiry).
    pub theta: f64,
}

/// Validate common Black-76 inputs.
///
/// Returns `Err(PricingError::InvalidInput)` if any parameter is out of range.
fn validate_inputs(forward: f64, strike: f64, sigma: f64, t: f64) -> Result<(), PricingError> {
    if forward <= 0.0 {
        return Err(PricingError::InvalidInput(
            "forward must be positive".into(),
        ));
    }
    if strike <= 0.0 {
        return Err(PricingError::InvalidInput("strike must be positive".into()));
    }
    if sigma < 0.0 {
        return Err(PricingError::InvalidInput(
            "sigma must be non-negative".into(),
        ));
    }
    if t < 0.0 {
        return Err(PricingError::InvalidInput(
            "time to expiry must be non-negative".into(),
        ));
    }
    Ok(())
}

/// Compute d1 and d2 for the Black-76 model.
///
/// d1 = [ln(F/K) + 0.5 * sigma^2 * T] / (sigma * sqrt(T))
/// d2 = d1 - sigma * sqrt(T)
#[inline]
fn d1_d2(forward: f64, strike: f64, sigma: f64, t: f64) -> (f64, f64) {
    let sqrt_t = t.sqrt();
    let sigma_sqrt_t = sigma * sqrt_t;
    let d1 = ((forward / strike).ln() + 0.5 * sigma * sigma * t) / sigma_sqrt_t;
    let d2 = d1 - sigma_sqrt_t;
    (d1, d2)
}

/// Compute the undiscounted Black-76 option price.
///
/// # Arguments
/// * `forward` - Forward price (must be positive)
/// * `strike` - Strike price (must be positive)
/// * `sigma` - Volatility (must be non-negative)
/// * `t` - Time to expiry in years (must be non-negative)
/// * `q` - Option type: +1 for call, -1 for put
///
/// # Returns
/// Undiscounted option price, or `PricingError` for invalid inputs.
///
/// # Examples
/// ```
/// # use essvi::pricing::black76::price;
/// let c = price(100.0, 100.0, 0.20, 1.0, 1).unwrap();
/// assert!((c - 7.965567455405804).abs() < 1e-10);
/// ```
pub fn price(forward: f64, strike: f64, sigma: f64, t: f64, q: i32) -> Result<f64, PricingError> {
    validate_inputs(forward, strike, sigma, t)?;

    if q != 1 && q != -1 {
        return Err(PricingError::InvalidInput(
            "q must be +1 (call) or -1 (put)".into(),
        ));
    }

    // Degenerate cases
    if sigma == 0.0 || t == 0.0 {
        let intrinsic = if q == 1 {
            (forward - strike).max(0.0)
        } else {
            (strike - forward).max(0.0)
        };
        return Ok(intrinsic);
    }

    let (d1, d2) = d1_d2(forward, strike, sigma, t);
    let qf = q as f64;
    Ok(qf * (forward * norm_cdf(qf * d1) - strike * norm_cdf(qf * d2)))
}

/// Compute the undiscounted Black-76 delta.
///
/// Delta = q * N(q * d1)
///
/// # Arguments
/// * `forward` - Forward price (must be positive)
/// * `strike` - Strike price (must be positive)
/// * `sigma` - Volatility (must be non-negative)
/// * `t` - Time to expiry in years (must be non-negative)
/// * `q` - Option type: +1 for call, -1 for put
///
/// # Examples
/// ```
/// # use essvi::pricing::black76::delta;
/// let d = delta(100.0, 100.0, 0.20, 1.0, 1).unwrap();
/// assert!((d - 0.5398278372770290).abs() < 1e-10);
/// ```
pub fn delta(forward: f64, strike: f64, sigma: f64, t: f64, q: i32) -> Result<f64, PricingError> {
    validate_inputs(forward, strike, sigma, t)?;

    if q != 1 && q != -1 {
        return Err(PricingError::InvalidInput(
            "q must be +1 (call) or -1 (put)".into(),
        ));
    }

    if sigma == 0.0 || t == 0.0 {
        let qf = q as f64;
        if forward > strike {
            return Ok(if q == 1 { 1.0 } else { 0.0 });
        } else if forward < strike {
            return Ok(if q == 1 { 0.0 } else { -1.0 });
        } else {
            return Ok(qf * 0.5);
        }
    }

    let (d1, _d2) = d1_d2(forward, strike, sigma, t);
    let qf = q as f64;
    Ok(qf * norm_cdf(qf * d1))
}

/// Compute the undiscounted Black-76 gamma.
///
/// Gamma = n(d1) / (F * sigma * sqrt(T))
///
/// where n(x) is the standard normal PDF.
///
/// Gamma is the same for calls and puts.
///
/// # Examples
/// ```
/// # use essvi::pricing::black76::gamma;
/// let g = gamma(100.0, 100.0, 0.20, 1.0).unwrap();
/// assert!(g > 0.0);
/// ```
pub fn gamma(forward: f64, strike: f64, sigma: f64, t: f64) -> Result<f64, PricingError> {
    validate_inputs(forward, strike, sigma, t)?;

    if sigma == 0.0 || t == 0.0 {
        return Ok(0.0);
    }

    let sqrt_t = t.sqrt();
    let sigma_sqrt_t = sigma * sqrt_t;
    let (d1, _d2) = d1_d2(forward, strike, sigma, t);
    Ok(norm_pdf(d1) / (forward * sigma_sqrt_t))
}

/// Compute the undiscounted Black-76 vega.
///
/// Vega = F * sqrt(T) * n(d1)
///
/// Vega is the same for calls and puts.
///
/// # Examples
/// ```
/// # use essvi::pricing::black76::vega;
/// let v = vega(100.0, 100.0, 0.20, 1.0).unwrap();
/// assert!(v > 0.0);
/// ```
pub fn vega(forward: f64, strike: f64, sigma: f64, t: f64) -> Result<f64, PricingError> {
    validate_inputs(forward, strike, sigma, t)?;

    if sigma == 0.0 || t == 0.0 {
        return Ok(0.0);
    }

    let sqrt_t = t.sqrt();
    let (d1, _d2) = d1_d2(forward, strike, sigma, t);
    Ok(forward * sqrt_t * norm_pdf(d1))
}

/// Compute the undiscounted Black-76 theta.
///
/// Theta (for calls) = -F * n(d1) * sigma / (2 * sqrt(T))
///
/// This is the derivative with respect to time-to-expiry (not calendar time),
/// so theta is typically negative (option loses value as time passes).
///
/// Theta is the same for calls and puts in the Black-76 model (no drift term
/// difference since both forward and strike are undiscounted).
///
/// # Examples
/// ```
/// # use essvi::pricing::black76::theta;
/// let th = theta(100.0, 100.0, 0.20, 1.0).unwrap();
/// assert!(th < 0.0);  // time decay
/// ```
pub fn theta(forward: f64, strike: f64, sigma: f64, t: f64) -> Result<f64, PricingError> {
    validate_inputs(forward, strike, sigma, t)?;

    if sigma == 0.0 || t == 0.0 {
        return Ok(0.0);
    }

    let sqrt_t = t.sqrt();
    let (d1, _d2) = d1_d2(forward, strike, sigma, t);
    // dC/dT = F * n(d1) * sigma / (2*sqrt(T))
    // but theta is -dC/dt = -dC/dT (since t = T - current_time, dT = -dt)
    // Actually for Black-76 with T as time-to-expiry:
    // Theta = -F * n(d1) * sigma / (2*sqrt(T))
    Ok(-forward * norm_pdf(d1) * sigma / (2.0 * sqrt_t))
}

/// Compute the discounted Black-76 option price.
///
/// Equivalent to `discount_factor * price(forward, strike, sigma, t, q)`.
///
/// # Arguments
/// * `forward` - Forward price
/// * `strike` - Strike price
/// * `sigma` - Volatility
/// * `t` - Time to expiry
/// * `q` - +1 for call, -1 for put
/// * `discount_factor` - Present value factor (e.g., exp(-r*T))
///
/// # Examples
/// ```
/// # use essvi::pricing::black76::discounted_price;
/// let df = (-0.05_f64 * 1.0).exp();  // 5% rate, 1 year
/// let p = discounted_price(100.0, 100.0, 0.20, 1.0, 1, df).unwrap();
/// let undiscounted = essvi::pricing::black76::price(100.0, 100.0, 0.20, 1.0, 1).unwrap();
/// assert!((p - df * undiscounted).abs() < 1e-14);
/// ```
pub fn discounted_price(
    forward: f64,
    strike: f64,
    sigma: f64,
    t: f64,
    q: i32,
    discount_factor: f64,
) -> Result<f64, PricingError> {
    if discount_factor < 0.0 || discount_factor > 1.0 {
        return Err(PricingError::InvalidInput(
            "discount factor must be in [0, 1]".into(),
        ));
    }
    let undiscounted = price(forward, strike, sigma, t, q)?;
    Ok(discount_factor * undiscounted)
}

/// Compute all greeks in a single call.
///
/// More efficient than calling individual greek functions separately,
/// since d1, d2, and n(d1) are computed only once.
///
/// # Arguments
/// * `forward` - Forward price (must be positive)
/// * `strike` - Strike price (must be positive)
/// * `sigma` - Volatility (must be non-negative)
/// * `t` - Time to expiry in years (must be non-negative)
/// * `q` - Option type: +1 for call, -1 for put
///
/// # Returns
/// A [`Greeks`] struct containing price, delta, gamma, vega, and theta.
///
/// # Examples
/// ```
/// # use essvi::pricing::black76::greeks;
/// let g = greeks(100.0, 100.0, 0.20, 1.0, 1).unwrap();
/// assert!(g.price > 0.0);
/// assert!(g.delta > 0.0 && g.delta < 1.0);
/// assert!(g.gamma > 0.0);
/// assert!(g.vega > 0.0);
/// assert!(g.theta < 0.0);
/// ```
pub fn greeks(
    forward: f64,
    strike: f64,
    sigma: f64,
    t: f64,
    q: i32,
) -> Result<Greeks, PricingError> {
    validate_inputs(forward, strike, sigma, t)?;

    if q != 1 && q != -1 {
        return Err(PricingError::InvalidInput(
            "q must be +1 (call) or -1 (put)".into(),
        ));
    }

    // Degenerate cases
    if sigma == 0.0 || t == 0.0 {
        let qf = q as f64;
        let intrinsic = if q == 1 {
            (forward - strike).max(0.0)
        } else {
            (strike - forward).max(0.0)
        };
        let delta_val = if forward > strike {
            if q == 1 { 1.0 } else { 0.0 }
        } else if forward < strike {
            if q == 1 { 0.0 } else { -1.0 }
        } else {
            qf * 0.5
        };
        return Ok(Greeks {
            price: intrinsic,
            delta: delta_val,
            gamma: 0.0,
            vega: 0.0,
            theta: 0.0,
        });
    }

    let qf = q as f64;
    let sqrt_t = t.sqrt();
    let sigma_sqrt_t = sigma * sqrt_t;
    let (d1, d2) = d1_d2(forward, strike, sigma, t);
    let n_d1 = norm_pdf(d1);
    let cdf_qd1 = norm_cdf(qf * d1);
    let cdf_qd2 = norm_cdf(qf * d2);

    let price_val = qf * (forward * cdf_qd1 - strike * cdf_qd2);
    let delta_val = qf * cdf_qd1;
    let gamma_val = n_d1 / (forward * sigma_sqrt_t);
    let vega_val = forward * sqrt_t * n_d1;
    let theta_val = -forward * n_d1 * sigma / (2.0 * sqrt_t);

    Ok(Greeks {
        price: price_val,
        delta: delta_val,
        gamma: gamma_val,
        vega: vega_val,
        theta: theta_val,
    })
}
