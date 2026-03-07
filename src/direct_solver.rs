/// Direct SSVI solver: 4D Nelder-Mead over (theta, eta, gamma, rho) with algebraic no-arb barrier.
///
/// Unlike `crude_solver` which includes butterfly density penalty in the objective,
/// this module uses only the algebraic condition eta*(1+|rho|) <= 2 as a hard barrier.
/// The objective is pure SSE on total variance. Butterfly arbitrage is validated
/// post-hoc via `validate_butterfly` rather than penalized during optimization.
///
/// Multi-start sweep over initial (rho, eta) values avoids local minima in the
/// 4D parameter landscape.

use crate::model::ssvi;
use crate::solver::nelder_mead::{NelderMeadConfig, nelder_mead_bounded};

/// Configuration for the direct SSVI calibrator.
///
/// Collects parameter bounds, multi-start grid settings, Nelder-Mead tolerances,
/// and calendar penalty weight into one struct. Unlike `CrudeCalibConfig`, there
/// is no butterfly penalty weight or penalty grid — the objective is pure SSE.
#[derive(Debug, Clone)]
pub struct DirectCalibConfig {
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

    // ── Multi-start grid ────────────────────────────────────
    /// Initial ρ values for multi-start sweep.
    pub rho_starts: Vec<f64>,
    /// Initial η values for multi-start sweep.
    pub eta_starts: Vec<f64>,

    // ── Calendar spread penalty ──────────────────────────────
    /// Penalty weight for calendar spread (theta monotonicity) violations.
    /// Used by `calibrate_slice_with_prev` to penalize theta < prev_theta.
    pub lambda_calendar: f64,
}

impl Default for DirectCalibConfig {
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

            rho_starts: vec![-0.7, -0.3, 0.0, 0.3],
            eta_starts: vec![0.3, 0.8, 1.3],

            lambda_calendar: 10.0,
        }
    }
}

/// Result of direct SSVI calibration for a single slice.
#[derive(Debug, Clone)]
pub struct DirectCalibResult {
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

/// Input data for one volatility slice in a multi-slice calibration.
#[derive(Debug, Clone)]
pub struct SliceInput<'a> {
    /// Time to expiry.
    pub t: f64,
    /// Log-moneyness values for this slice.
    pub k: &'a [f64],
    /// Market total variance values for this slice.
    pub w: &'a [f64],
}

/// Result of butterfly arbitrage validation at a single point.
#[derive(Debug, Clone)]
pub struct ButterflyResult {
    /// Log-moneyness point.
    pub k: f64,
    /// Butterfly density g(k) value.
    pub g: f64,
    /// Whether g(k) >= 0 (no arbitrage) at this point.
    pub valid: bool,
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

/// Validate butterfly arbitrage absence for fitted SSVI parameters.
///
/// Evaluates the butterfly density g(k) at each point in `k_grid` and returns
/// per-point results indicating whether g(k) >= 0 (no arbitrage).
///
/// This is a post-hoc check — the direct solver does not include butterfly
/// density in its objective. Use this to verify fitted parameters.
pub fn validate_butterfly(
    theta: f64,
    eta: f64,
    gamma: f64,
    rho: f64,
    k_grid: &[f64],
) -> Vec<ButterflyResult> {
    k_grid
        .iter()
        .map(|&k| {
            let g = butterfly_density(k, theta, eta, gamma, rho);
            ButterflyResult {
                k,
                g,
                valid: g >= 0.0,
            }
        })
        .collect()
}

// ── Calibration ─────────────────────────────────────────────

/// Calibrate SSVI parameters to a single volatility slice via 4D Nelder-Mead.
///
/// Directly optimizes all four parameters (θ, η, γ, ρ) without implicit θ solve.
/// The objective is pure SSE on total variance:
///   f(θ, η, γ, ρ) = Σ(w_model - w_market)²
///
/// The coupled no-arb condition η·(1 + |ρ|) ≤ 2 is enforced as a hard barrier.
/// No butterfly density term is included in the objective — use `validate_butterfly`
/// for post-hoc arbitrage checking.
///
/// Multi-start sweep over initial (ρ, η) explores the parameter space to avoid
/// local minima. The best result (lowest SSE) is returned.
pub fn calibrate_slice(
    k_slice: &[f64],
    w_market: &[f64],
    config: &DirectCalibConfig,
) -> DirectCalibResult {
    calibrate_slice_with_prev(k_slice, w_market, config, None)
}

/// Core calibration logic for a single slice, with optional calendar spread penalty.
///
/// When `prev_theta` is `Some(prev)`, the objective includes:
///   lambda_calendar · max(0, prev - θ)²
/// to penalize θ values below the previous slice's fitted θ.
pub fn calibrate_slice_with_prev(
    k_slice: &[f64],
    w_market: &[f64],
    config: &DirectCalibConfig,
    prev_theta: Option<f64>,
) -> DirectCalibResult {
    let lambda_cal = config.lambda_calendar;

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

        // SSE on total variance (pure — no butterfly penalty)
        let w_model = ssvi::total_variance_slice(k_slice, theta, eta, gamma, rho);
        let sse: f64 = w_model
            .iter()
            .zip(w_market.iter())
            .map(|(m, mkt)| (m - mkt).powi(2))
            .sum();

        let mut total = sse;

        // Calendar spread penalty: penalize theta below previous slice's theta
        if let Some(prev) = prev_theta {
            let shortfall = (prev - theta).max(0.0);
            total += lambda_cal * shortfall * shortfall;
        }

        total
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

    // Initial theta estimate from median of w_market, biased by prev_theta if available
    let mut w_sorted: Vec<f64> = w_market.to_vec();
    w_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_w = if w_sorted.is_empty() {
        0.04
    } else {
        w_sorted[w_sorted.len() / 2]
    };
    let theta_init = if let Some(prev) = prev_theta {
        prev.max(median_w)
    } else {
        median_w
    }
    .clamp(lb[0], ub[0]);

    // Multi-start: sweep initial rho and eta to avoid local minima.
    let mut best_f = f64::INFINITY;
    let mut best_res = None;

    for &rho_init in &config.rho_starts {
        for &eta_init in &config.eta_starts {
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

    DirectCalibResult {
        theta: res.x[0],
        eta: res.x[1],
        gamma: res.x[2],
        rho: res.x[3],
        sse,
        converged: res.converged,
        iterations: res.iterations,
    }
}
