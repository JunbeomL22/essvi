/// SSVI calibration: solve θ implicitly, then minimize ||W - tv||².

use crate::solver::nelder_mead::{nelder_mead_bounded, NelderMeadConfig, NelderMeadResult};
use crate::model::ssvi;
use std::fmt;

/// Configuration for the SSVI calibration pipeline.
///
/// Collects all tuning knobs — parameter bounds, rho grid settings,
/// solver tolerances, and calendar penalty defaults — into one struct
/// so callers can override any subset while keeping sensible defaults.
#[derive(Debug, Clone)]
pub struct CalibrationConfig {
    // ── Parameter bounds ───────────────────────────────────────
    /// Lower bound for η (must be > 0).
    pub eta_lower: f64,
    /// Upper bound for η (must be < 2 to satisfy no-arb with ρ near 0).
    pub eta_upper: f64,
    /// Lower bound for γ (must be > 0).
    pub gamma_lower: f64,
    /// Upper bound for γ (must be < 1).
    pub gamma_upper: f64,
    /// Lower bound for ρ (must be > -1).
    pub rho_lower: f64,
    /// Upper bound for ρ (must be < 1).
    pub rho_upper: f64,

    // ── Rho grid sweep ─────────────────────────────────────────
    /// Number of steps in the ρ grid sweep (n_rho+1 points evaluated).
    pub n_rho: usize,
    /// Start of ρ sweep range.
    pub rho_sweep_start: f64,
    /// End of ρ sweep range.
    pub rho_sweep_end: f64,

    // ── solve_theta Newton solver ──────────────────────────────
    /// Maximum Newton iterations for θ solve.
    pub theta_max_iter: usize,
    /// Convergence tolerance for θ solve residual.
    pub theta_tol: f64,

    // ── Nelder-Mead optimizer ──────────────────────────────────
    /// Configuration passed to the Nelder-Mead optimizer.
    pub nelder_mead: NelderMeadConfig,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            eta_lower: 1e-6,
            eta_upper: 2.0 - 1e-6,
            gamma_lower: 1e-6,
            gamma_upper: 1.0 - 1e-6,
            rho_lower: -0.999,
            rho_upper: 0.999,

            n_rho: 20,
            rho_sweep_start: -0.95,
            rho_sweep_end: 0.95,

            theta_max_iter: 20,
            theta_tol: 1e-14,

            nelder_mead: NelderMeadConfig::default(),
        }
    }
}

/// Error type for calibration failures.
///
/// Provides distinct variants so callers can distinguish failure modes
/// and respond appropriately (e.g., retry with different initial guess,
/// widen bounds, or report specific diagnostics).
#[derive(Debug, Clone)]
pub enum CalibError {
    /// Newton iteration for θ produced a non-positive value.
    NonPositiveTheta,
    /// Newton iteration for θ encountered a near-zero derivative (∂w/∂θ ≈ 0).
    ZeroDerivative,
    /// Newton iteration for θ did not converge within the iteration limit.
    ThetaDivergence,
    /// The Nelder-Mead optimizer did not converge within its iteration budget.
    NonConvergence,
}

impl fmt::Display for CalibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalibError::NonPositiveTheta => write!(f, "theta went non-positive during Newton iteration"),
            CalibError::ZeroDerivative => write!(f, "zero derivative encountered during theta solve"),
            CalibError::ThetaDivergence => write!(f, "theta solve did not converge"),
            CalibError::NonConvergence => write!(f, "calibration optimizer did not converge"),
        }
    }
}

impl std::error::Error for CalibError {}

/// Solve θ from the ATM consistency equation w(k*, θ) = θ*
/// using Newton's method with initial guess θ₀ = θ*.
///
/// Each Newton step:
///   φ  = η / (θ^γ · (1+θ)^(1-γ))
///   S  = sqrt((φ·k* + ρ)² + (1 - ρ²))
///   w  = (θ/2) · (1 + ρ·φ·k* + S)
///   ∂w = w/θ - (γ+θ)·φ·k* / (2(1+θ)) · (ρ + (φ·k* + ρ)/S)
///   θ ← θ - (w - θ*) / ∂w
///
/// Typically converges in 2–3 iterations.
pub fn solve_theta(
    eta: f64,
    gamma: f64,
    rho: f64,
    theta_star: f64,
    k_star: f64,
    config: &CalibrationConfig,
) -> Result<f64, CalibError> {
    if k_star.abs() < 1e-15 {
        return Ok(theta_star);
    }

    let mut theta = theta_star; // initial guess
    let max_iter = config.theta_max_iter;
    let tol = config.theta_tol;

    for _ in 0..max_iter {
        if theta <= 0.0 {
            return Err(CalibError::NonPositiveTheta);
        }

        let phi = ssvi::phi(theta, eta, gamma);
        let pk = phi * k_star;
        let s = ((pk + rho).powi(2) + (1.0 - rho * rho)).sqrt();
        let w = 0.5 * theta * (1.0 + rho * pk + s);

        let residual = w - theta_star;
        if residual.abs() < tol {
            return Ok(theta);
        }

        // ∂w/∂θ
        let dw = w / theta
            - (gamma + theta) * phi * k_star / (2.0 * (1.0 + theta))
                * (rho + (pk + rho) / s);

        if dw.abs() < 1e-30 {
            return Err(CalibError::ZeroDerivative);
        }

        theta -= residual / dw;
    }

    // Check final convergence
    if theta > 0.0 {
        let phi = ssvi::phi(theta, eta, gamma);
        let pk = phi * k_star;
        let s = ((pk + rho).powi(2) + (1.0 - rho * rho)).sqrt();
        let w = 0.5 * theta * (1.0 + rho * pk + s);
        if (w - theta_star).abs() < tol * 100.0 {
            return Ok(theta);
        }
    }
    Err(CalibError::ThetaDivergence)
}

/// Weighted squared error between model and market total variances.
fn weighted_squared_error(model: &[f64], market: &[f64], weights: Option<&[f64]>) -> f64 {
    match weights {
        Some(w) => model
            .iter()
            .zip(market.iter())
            .zip(w.iter())
            .map(|((m, mkt), &wt)| wt * (m - mkt).powi(2))
            .sum(),
        None => model
            .iter()
            .zip(market.iter())
            .map(|(m, mkt)| (m - mkt).powi(2))
            .sum(),
    }
}

#[derive(Debug, Clone)]
pub struct CalibrationInput<'a> {
    pub k_slice: &'a [f64],
    pub w_market: &'a [f64],
    pub theta_star: f64,
    pub k_star: f64,
    pub weights: Option<&'a [f64]>,
}

#[derive(Debug, Clone)]
pub struct CalibrationResult {
    pub eta: f64,
    pub gamma: f64,
    pub rho: f64,
    pub theta: f64,
    pub optimizer: NelderMeadResult,
}

impl CalibrationResult {
    /// Compute the SSVI φ(θ) value from the calibrated parameters.
    ///
    /// φ(θ) = η / (θ^γ · (1+θ)^(1-γ))
    pub fn phi(&self) -> f64 {
        ssvi::phi(self.theta, self.eta, self.gamma)
    }

    /// Compute how close the calibrated parameters are to the no-arbitrage boundary.
    ///
    /// Returns η·(1 + |ρ|). The no-arbitrage condition requires this to be ≤ 2.
    /// Values approaching 2.0 indicate the optimizer is constrained by the
    /// no-arbitrage bound, which may degrade fit quality.
    pub fn no_arb_usage(&self) -> f64 {
        self.eta * (1.0 + self.rho.abs())
    }
}

/// Calibrate SSVI parameters (η, γ, ρ) to a single slice.
///
/// Uses ρ-grid sweep + 2D Nelder-Mead on (η, γ) to avoid local minima,
/// as recommended for SSVI calibration. The equality constraint (ATM
/// consistency) is eliminated by solving θ implicitly inside the objective.
pub fn calibrate(input: &CalibrationInput, config: &CalibrationConfig) -> Result<CalibrationResult, CalibError> {
    let lb_eg = [config.eta_lower, config.gamma_lower];
    let ub_eg = [config.eta_upper, config.gamma_upper];

    let mut best_f = f64::INFINITY;
    let mut best_x = [0.5, 0.5, 0.0];

    // ρ grid: sweep from rho_sweep_start to rho_sweep_end in n_rho steps
    let n_rho = config.n_rho;
    let rho_range = config.rho_sweep_end - config.rho_sweep_start;
    for i in 0..=n_rho {
        let rho = config.rho_sweep_start + (i as f64) * rho_range / (n_rho as f64);

        let objective_2d = |x: &[f64]| -> f64 {
            let eta = x[0];
            let gamma = x[1];

            if !ssvi::no_arbitrage_satisfied(eta, rho) {
                return 1e10;
            }

            let theta = match solve_theta(eta, gamma, rho, input.theta_star, input.k_star, config) {
                Ok(t) => t,
                Err(_) => return 1e10,
            };

            let w_model = ssvi::total_variance_slice(input.k_slice, theta, eta, gamma, rho);
            weighted_squared_error(&w_model, input.w_market, input.weights)
        };

        let res = nelder_mead_bounded(objective_2d, &[0.5, 0.5], &lb_eg, &ub_eg, &config.nelder_mead);
        if res.f < best_f {
            best_f = res.f;
            best_x = [res.x[0], res.x[1], rho];
        }
    }

    // Polish: 3D optimization starting from best grid point
    let objective_3d = |x: &[f64]| -> f64 {
        let eta = x[0];
        let gamma = x[1];
        let rho = x[2];

        if !ssvi::no_arbitrage_satisfied(eta, rho) {
            return 1e10;
        }

        let theta = match solve_theta(eta, gamma, rho, input.theta_star, input.k_star, config) {
            Ok(t) => t,
            Err(_) => return 1e10,
        };

        let w_model = ssvi::total_variance_slice(input.k_slice, theta, eta, gamma, rho);
        weighted_squared_error(&w_model, input.w_market, input.weights)
    };

    let lb_3d = [config.eta_lower, config.gamma_lower, config.rho_lower];
    let ub_3d = [config.eta_upper, config.gamma_upper, config.rho_upper];
    let res = nelder_mead_bounded(objective_3d, &best_x, &lb_3d, &ub_3d, &config.nelder_mead);

    let eta = res.x[0];
    let gamma = res.x[1];
    let rho = res.x[2];
    let theta = solve_theta(eta, gamma, rho, input.theta_star, input.k_star, config)?;

    Ok(CalibrationResult {
        eta,
        gamma,
        rho,
        theta,
        optimizer: res,
    })
}

/// Previous slice info for calendar arbitrage penalty.
#[derive(Debug, Clone)]
pub struct PrevSlice {
    pub theta: f64,
    pub eta: f64,
    pub gamma: f64,
    pub rho: f64,
}

/// Calendar arbitrage penalty: sum of max(0, w_prev(k) - w_cur(k))^2
/// evaluated at the given penalty sample points.
fn calendar_penalty(prev: &PrevSlice, theta: f64, eta: f64, gamma: f64, rho: f64, k_penalty: &[f64]) -> f64 {
    k_penalty.iter().map(|&k| {
        let w_prev = ssvi::total_variance(k, prev.theta, prev.eta, prev.gamma, prev.rho);
        let w_cur = ssvi::total_variance(k, theta, eta, gamma, rho);
        let violation = (w_prev - w_cur).max(0.0);
        violation * violation
    }).sum()
}

/// Calibrate SSVI with calendar arbitrage penalty.
///
/// Same as `calibrate` but adds λ · Σ max(0, w_prev(k) - w(k))² to the objective.
/// `k_penalty` are the sample points for checking calendar arbitrage.
pub fn calibrate_with_calendar_penalty(
    input: &CalibrationInput,
    config: &CalibrationConfig,
    prev: &PrevSlice,
    k_penalty: &[f64],
    lambda: f64,
    init: &[f64; 3], // initial [eta, gamma, rho] from unconstrained fit
) -> Result<CalibrationResult, CalibError> {
    let objective_3d = |x: &[f64]| -> f64 {
        let eta = x[0];
        let gamma = x[1];
        let rho = x[2];

        if !ssvi::no_arbitrage_satisfied(eta, rho) {
            return 1e10;
        }

        let theta = match solve_theta(eta, gamma, rho, input.theta_star, input.k_star, config) {
            Ok(t) => t,
            Err(_) => return 1e10,
        };

        let w_model = ssvi::total_variance_slice(input.k_slice, theta, eta, gamma, rho);
        let fit_err = weighted_squared_error(&w_model, input.w_market, input.weights);
        let penalty = calendar_penalty(prev, theta, eta, gamma, rho, k_penalty);
        fit_err + lambda * penalty
    };

    let lb_3d = [config.eta_lower, config.gamma_lower, config.rho_lower];
    let ub_3d = [config.eta_upper, config.gamma_upper, config.rho_upper];
    let res = nelder_mead_bounded(objective_3d, init, &lb_3d, &ub_3d, &config.nelder_mead);

    let eta = res.x[0];
    let gamma = res.x[1];
    let rho = res.x[2];
    let theta = solve_theta(eta, gamma, rho, input.theta_star, input.k_star, config)?;

    Ok(CalibrationResult {
        eta,
        gamma,
        rho,
        theta,
        optimizer: res,
    })
}
