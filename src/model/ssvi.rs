/// SSVI model: φ function and total variance w(k, θ).

/// φ(θ) = η / (θ^γ · (1+θ)^(1-γ))
#[inline]
pub fn phi(theta: f64, eta: f64, gamma: f64) -> f64 {
    eta / (theta.powf(gamma) * (1.0 + theta).powf(1.0 - gamma))
}

/// SSVI total variance for a single strike:
/// w(k, θ) = (θ/2) · {1 + ρ·φ(θ)·k + sqrt((φ(θ)·k + ρ)² + (1 - ρ²))}
#[inline]
pub fn total_variance(k: f64, theta: f64, eta: f64, gamma: f64, rho: f64) -> f64 {
    let p = phi(theta, eta, gamma);
    let pk = p * k;
    let disc = (pk + rho).powi(2) + (1.0 - rho * rho);
    0.5 * theta * (1.0 + rho * pk + disc.sqrt())
}

/// Compute total variance for a slice of log-moneyness values.
pub fn total_variance_slice(
    k_slice: &[f64],
    theta: f64,
    eta: f64,
    gamma: f64,
    rho: f64,
) -> Vec<f64> {
    k_slice
        .iter()
        .map(|&k| total_variance(k, theta, eta, gamma, rho))
        .collect()
}

/// No-arbitrage condition: η(1 + |ρ|) ≤ 2
#[inline]
pub fn no_arbitrage_satisfied(eta: f64, rho: f64) -> bool {
    eta * (1.0 + rho.abs()) <= 2.0
}
