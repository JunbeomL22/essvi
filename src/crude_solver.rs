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

            nelder_mead: NelderMeadConfig::default(),

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
