/// Crude SSVI solver: direct 4D Nelder-Mead over (θ, η, γ, ρ) with butterfly penalty.
///
/// Unlike `calibration::calibrate` which solves θ implicitly via Newton's method
/// and sweeps ρ on a grid, this module treats all four SSVI parameters as free
/// variables and optimizes them directly. No-arbitrage butterfly constraints are
/// enforced via a penalty term in the objective rather than by construction.

use crate::model::ssvi;
use crate::solver::nelder_mead::{NelderMeadConfig, nelder_mead_bounded};

/// Configuration for the crude SSVI calibrator.
///
/// Collects parameter bounds, Nelder-Mead tolerances, butterfly penalty weight,
/// and penalty evaluation grid settings into one struct.
#[derive(Debug, Clone)]
pub struct CrudeCalibConfig {
    // ── Theta bounds ────────────────────────────────────────
    /// Lower bound for θ (must be > 0).
    pub theta_lower: f64,
    /// Upper bound for θ.
    pub theta_upper: f64,

    // ── Parameter bounds (η, γ, ρ) ─────────────────────────
    /// Lower bound for η (must be > 0).
    pub eta_lower: f64,
    /// Upper bound for η.
    pub eta_upper: f64,
    /// Lower bound for γ (must be > 0).
    pub gamma_lower: f64,
    /// Upper bound for γ (must be < 1).
    pub gamma_upper: f64,
    /// Lower bound for ρ (must be > -1).
    pub rho_lower: f64,
    /// Upper bound for ρ (must be < 1).
    pub rho_upper: f64,

    // ── 4D Nelder-Mead config ───────────────────────────────
    /// Configuration passed to the bounded Nelder-Mead optimizer.
    pub nelder_mead: NelderMeadConfig,

    // ── Butterfly penalty ───────────────────────────────────
    /// Penalty weight for butterfly no-arbitrage violations.
    pub lambda: f64,

    // ── Penalty evaluation grid ─────────────────────────────
    /// Left edge of the penalty evaluation grid (log-moneyness).
    pub k_penalty_lo: f64,
    /// Right edge of the penalty evaluation grid (log-moneyness).
    pub k_penalty_hi: f64,
    /// Number of points in the penalty evaluation grid.
    pub k_penalty_n: usize,
}

impl Default for CrudeCalibConfig {
    fn default() -> Self {
        Self {
            theta_lower: 1e-6,
            theta_upper: 1.0,

            eta_lower: 1e-6,
            eta_upper: 2.0 - 1e-6,
            gamma_lower: 1e-6,
            gamma_upper: 1.0 - 1e-6,
            rho_lower: -0.999,
            rho_upper: 0.999,

            nelder_mead: NelderMeadConfig {
                max_iter: 5000,
                ..NelderMeadConfig::default()
            },

            lambda: 1.0,

            k_penalty_lo: -2.0,
            k_penalty_hi: 2.0,
            k_penalty_n: 50,
        }
    }
}

/// Result of crude SSVI calibration for a single slice.
#[derive(Debug, Clone)]
pub struct CrudeCalibResult {
    /// Calibrated ATM total variance.
    pub theta: f64,
    /// Calibrated η parameter.
    pub eta: f64,
    /// Calibrated γ parameter.
    pub gamma: f64,
    /// Calibrated ρ (correlation) parameter.
    pub rho: f64,
    /// Sum of squared errors on total variance.
    pub sse: f64,
    /// Whether the optimizer converged.
    pub converged: bool,
    /// Number of optimizer iterations used.
    pub iterations: usize,
}

// ── Butterfly arbitrage helpers ─────────────────────────────

/// Butterfly density g(k) for the SSVI smile.
///
/// The no-arbitrage butterfly condition requires g(k) >= 0 for all k, where:
///   g(k) = (1 - k·w'/(2w))² - (w')²/4 · (1/w + 1/4) + w''/2
///
/// Uses central finite differences for w' and w''.
fn butterfly_density(k: f64, theta: f64, eta: f64, gamma: f64, rho: f64) -> f64 {
    let w = ssvi::total_variance(k, theta, eta, gamma, rho);
    if w <= 0.0 {
        return 0.0;
    }

    let h = 1e-5;
    let w_plus = ssvi::total_variance(k + h, theta, eta, gamma, rho);
    let w_minus = ssvi::total_variance(k - h, theta, eta, gamma, rho);

    let w_prime = (w_plus - w_minus) / (2.0 * h);
    let w_double_prime = (w_plus - 2.0 * w + w_minus) / (h * h);

    let term1 = 1.0 - k * w_prime / (2.0 * w);
    let term2 = w_prime * w_prime / 4.0 * (1.0 / w + 0.25);

    term1 * term1 - term2 + w_double_prime / 2.0
}

/// Sum of squared butterfly violations across the penalty grid.
///
/// For each grid point where g(k) < 0, adds g(k)² to the penalty.
fn butterfly_penalty(theta: f64, eta: f64, gamma: f64, rho: f64, k_grid: &[f64]) -> f64 {
    k_grid
        .iter()
        .map(|&k| {
            let g = butterfly_density(k, theta, eta, gamma, rho);
            if g < 0.0 { g * g } else { 0.0 }
        })
        .sum()
}

// ── Calibration ─────────────────────────────────────────────

/// Calibrate SSVI parameters to a single volatility slice via 4D Nelder-Mead.
///
/// Directly optimizes all four parameters (θ, η, γ, ρ) without implicit θ solve.
/// The objective is SSE on total variance plus a butterfly no-arbitrage penalty:
///   f(θ, η, γ, ρ) = Σ(w_model - w_market)² + λ · butterfly_penalty
///
/// The coupled no-arb condition η·(1 + |ρ|) ≤ 2 is enforced as a hard barrier.
pub fn calibrate_slice(
    k_slice: &[f64],
    w_market: &[f64],
    config: &CrudeCalibConfig,
) -> CrudeCalibResult {
    // Build penalty evaluation grid
    let k_grid: Vec<f64> = if config.k_penalty_n <= 1 {
        vec![0.0]
    } else {
        (0..config.k_penalty_n)
            .map(|i| {
                config.k_penalty_lo
                    + (i as f64) * (config.k_penalty_hi - config.k_penalty_lo)
                        / (config.k_penalty_n - 1) as f64
            })
            .collect()
    };

    let lambda = config.lambda;

    // 4D objective: x = [theta, eta, gamma, rho]
    let objective = |x: &[f64]| -> f64 {
        let theta = x[0];
        let eta = x[1];
        let gamma = x[2];
        let rho = x[3];

        // Hard barrier: no-arb condition
        if !ssvi::no_arbitrage_satisfied(eta, rho) {
            return 1e10;
        }

        // SSE on total variance
        let w_model = ssvi::total_variance_slice(k_slice, theta, eta, gamma, rho);
        let sse: f64 = w_model
            .iter()
            .zip(w_market.iter())
            .map(|(m, mkt)| (m - mkt).powi(2))
            .sum();

        // Butterfly penalty
        let penalty = butterfly_penalty(theta, eta, gamma, rho, &k_grid);

        sse + lambda * penalty
    };

    // Bounds: [theta, eta, gamma, rho]
    let lb = [
        config.theta_lower,
        config.eta_lower,
        config.gamma_lower,
        config.rho_lower,
    ];
    let ub = [
        config.theta_upper,
        config.eta_upper,
        config.gamma_upper,
        config.rho_upper,
    ];

    // Initial theta estimate from median of w_market
    let mut w_sorted: Vec<f64> = w_market.to_vec();
    w_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let theta_init = if w_sorted.is_empty() {
        0.04
    } else {
        w_sorted[w_sorted.len() / 2]
    }
    .clamp(lb[0], ub[0]);

    // Multi-start: sweep initial rho and eta to avoid local minima.
    // The 4D landscape has ridges where (eta, gamma) trade off;
    // varying starting points helps the optimizer find the global basin.
    let rho_starts: [f64; 4] = [-0.7, -0.3, 0.0, 0.3];
    let eta_starts: [f64; 3] = [0.3, 0.8, 1.3];

    let mut best_f = f64::INFINITY;
    let mut best_res = None;

    for &rho_init in &rho_starts {
        for &eta_init in &eta_starts {
            let x0 = [
                theta_init,
                eta_init.clamp(lb[1], ub[1]),
                0.5_f64.clamp(lb[2], ub[2]),
                rho_init.clamp(lb[3], ub[3]),
            ];

            let res = nelder_mead_bounded(&objective, &x0, &lb, &ub, &config.nelder_mead);
            if res.f < best_f {
                best_f = res.f;
                best_res = Some(res);
            }
        }
    }

    let res = best_res.expect("at least one start point must run");

    // Compute final SSE (without penalty) for reporting
    let w_model = ssvi::total_variance_slice(k_slice, res.x[0], res.x[1], res.x[2], res.x[3]);
    let sse: f64 = w_model
        .iter()
        .zip(w_market.iter())
        .map(|(m, mkt)| (m - mkt).powi(2))
        .sum();

    CrudeCalibResult {
        theta: res.x[0],
        eta: res.x[1],
        gamma: res.x[2],
        rho: res.x[3],
        sse,
        converged: res.converged,
        iterations: res.iterations,
    }
}
