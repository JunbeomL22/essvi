/// SSVI calibration: solve θ implicitly, then minimize ||W - tv||².

use crate::solver::nelder_mead::{nelder_mead_bounded, NelderMeadConfig, NelderMeadResult};
use crate::model::ssvi;

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
) -> Option<f64> {
    if k_star.abs() < 1e-15 {
        return Some(theta_star);
    }

    let mut theta = theta_star; // initial guess
    let max_iter = 20;
    let tol = 1e-14;

    for _ in 0..max_iter {
        if theta <= 0.0 {
            return None;
        }

        let phi = ssvi::phi(theta, eta, gamma);
        let pk = phi * k_star;
        let s = ((pk + rho).powi(2) + (1.0 - rho * rho)).sqrt();
        let w = 0.5 * theta * (1.0 + rho * pk + s);

        let residual = w - theta_star;
        if residual.abs() < tol {
            return Some(theta);
        }

        // ∂w/∂θ
        let dw = w / theta
            - (gamma + theta) * phi * k_star / (2.0 * (1.0 + theta))
                * (rho + (pk + rho) / s);

        if dw.abs() < 1e-30 {
            return None;
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
            return Some(theta);
        }
    }
    None
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

/// Calibrate SSVI parameters (η, γ, ρ) to a single slice.
///
/// Uses ρ-grid sweep + 2D Nelder-Mead on (η, γ) to avoid local minima,
/// as recommended for SSVI calibration. The equality constraint (ATM
/// consistency) is eliminated by solving θ implicitly inside the objective.
pub fn calibrate(input: &CalibrationInput, config: &NelderMeadConfig) -> Option<CalibrationResult> {
    let lb_eg = [1e-6, 1e-6];
    let ub_eg = [2.0 - 1e-6, 1.0 - 1e-6];

    let mut best_f = f64::INFINITY;
    let mut best_x = [0.5, 0.5, 0.0];

    // ρ grid: sweep from -0.95 to 0.95 in 20 steps
    let n_rho = 20;
    for i in 0..=n_rho {
        let rho = -0.95 + (i as f64) * 1.9 / (n_rho as f64);

        let objective_2d = |x: &[f64]| -> f64 {
            let eta = x[0];
            let gamma = x[1];

            if !ssvi::no_arbitrage_satisfied(eta, rho) {
                return 1e10;
            }

            let theta = match solve_theta(eta, gamma, rho, input.theta_star, input.k_star) {
                Some(t) => t,
                None => return 1e10,
            };

            let w_model = ssvi::total_variance_slice(input.k_slice, theta, eta, gamma, rho);
            weighted_squared_error(&w_model, input.w_market, input.weights)
        };

        let res = nelder_mead_bounded(objective_2d, &[0.5, 0.5], &lb_eg, &ub_eg, config);
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

        let theta = match solve_theta(eta, gamma, rho, input.theta_star, input.k_star) {
            Some(t) => t,
            None => return 1e10,
        };

        let w_model = ssvi::total_variance_slice(input.k_slice, theta, eta, gamma, rho);
        weighted_squared_error(&w_model, input.w_market, input.weights)
    };

    let lb_3d = [1e-6, 1e-6, -0.999];
    let ub_3d = [2.0 - 1e-6, 1.0 - 1e-6, 0.999];
    let res = nelder_mead_bounded(objective_3d, &best_x, &lb_3d, &ub_3d, config);

    let eta = res.x[0];
    let gamma = res.x[1];
    let rho = res.x[2];
    let theta = solve_theta(eta, gamma, rho, input.theta_star, input.k_star)?;

    Some(CalibrationResult {
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
    config: &NelderMeadConfig,
    prev: &PrevSlice,
    k_penalty: &[f64],
    lambda: f64,
    init: &[f64; 3], // initial [eta, gamma, rho] from unconstrained fit
) -> Option<CalibrationResult> {
    let objective_3d = |x: &[f64]| -> f64 {
        let eta = x[0];
        let gamma = x[1];
        let rho = x[2];

        if !ssvi::no_arbitrage_satisfied(eta, rho) {
            return 1e10;
        }

        let theta = match solve_theta(eta, gamma, rho, input.theta_star, input.k_star) {
            Some(t) => t,
            None => return 1e10,
        };

        let w_model = ssvi::total_variance_slice(input.k_slice, theta, eta, gamma, rho);
        let fit_err = weighted_squared_error(&w_model, input.w_market, input.weights);
        let penalty = calendar_penalty(prev, theta, eta, gamma, rho, k_penalty);
        fit_err + lambda * penalty
    };

    let lb_3d = [1e-6, 1e-6, -0.999];
    let ub_3d = [2.0 - 1e-6, 1.0 - 1e-6, 0.999];
    let res = nelder_mead_bounded(objective_3d, init, &lb_3d, &ub_3d, config);

    let eta = res.x[0];
    let gamma = res.x[1];
    let rho = res.x[2];
    let theta = solve_theta(eta, gamma, rho, input.theta_star, input.k_star)?;

    Some(CalibrationResult {
        eta,
        gamma,
        rho,
        theta,
        optimizer: res,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a synthetic 20-point smile slice from known SSVI parameters,
    /// then calibrate back and verify recovery.
    fn make_sample_slice() -> (Vec<f64>, Vec<f64>, f64, f64) {
        let true_eta = 0.8;
        let true_gamma = 0.4;
        let true_rho = -0.35;
        let theta_star = 0.04; // ATM total variance (σ²·T ≈ 20%² · 1Y)
        let k_star = -0.01;    // Slightly off-ATM to break η/γ degeneracy

        // Solve true θ
        let true_theta = solve_theta(true_eta, true_gamma, true_rho, theta_star, k_star)
            .expect("true theta must solve");

        // 20 log-moneyness points from -0.5 to +0.5
        let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
        let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

        (k_slice, w_market, theta_star, k_star)
    }

    #[test]
    fn solve_theta_basic() {
        let theta = solve_theta(0.8, 0.4, -0.35, 0.04, 0.0);
        assert!(theta.is_some());
        let t = theta.unwrap();
        // With k*=0, the equation simplifies to θ = θ*
        assert!((t - 0.04).abs() < 1e-10);
    }

    #[test]
    fn solve_theta_nonzero_kstar() {
        let theta = solve_theta(0.8, 0.4, -0.35, 0.04, 0.01);
        assert!(theta.is_some());
        let t = theta.unwrap();
        // Verify w(k*, θ) = θ*
        let w = ssvi::total_variance(0.01, t, 0.8, 0.4, -0.35);
        assert!((w - 0.04).abs() < 1e-12, "w={}, θ*=0.04, diff={}", w, (w - 0.04).abs());
    }

    #[test]
    fn calibrate_recovers_parameters() {
        let (k_slice, w_market, theta_star, k_star) = make_sample_slice();

        let true_theta = solve_theta(0.8, 0.4, -0.35, theta_star, k_star).unwrap();
        let true_phi = ssvi::phi(true_theta, 0.8, 0.4);

        let input = CalibrationInput {
            k_slice: &k_slice,
            w_market: &w_market,
            theta_star,
            k_star,
            weights: None,
        };

        let res = calibrate(&input, &NelderMeadConfig::default())
            .expect("calibration must succeed");

        assert!(res.optimizer.converged, "optimizer did not converge");
        assert!(res.optimizer.f < 1e-20, "residual too large: {}", res.optimizer.f);

        // ρ is directly identifiable from skew shape
        assert!((res.rho - (-0.35)).abs() < 1e-3, "rho: {}", res.rho);

        // φ(θ) is identifiable (controls smile curvature), though η/γ individually are not
        let recovered_phi = ssvi::phi(res.theta, res.eta, res.gamma);
        assert!(
            (recovered_phi - true_phi).abs() / true_phi < 1e-3,
            "phi: {} vs true {}", recovered_phi, true_phi
        );

        // Verify the model reproduces market prices
        let w_fit = ssvi::total_variance_slice(&k_slice, res.theta, res.eta, res.gamma, res.rho);
        let max_err: f64 = w_fit.iter().zip(w_market.iter())
            .map(|(m, w)| (m - w).abs())
            .fold(0.0_f64, f64::max);
        assert!(max_err < 1e-10, "max pointwise error: {}", max_err);
    }

    #[test]
    fn calibrate_with_nonzero_kstar() {
        let true_eta = 0.6;
        let true_gamma = 0.3;
        let true_rho = -0.5;
        let theta_star = 0.05;
        let k_star = -0.02;

        let true_theta = solve_theta(true_eta, true_gamma, true_rho, theta_star, k_star)
            .expect("true theta must solve");

        let k_slice: Vec<f64> = (0..20).map(|i| -0.4 + (i as f64) * 0.04).collect();
        let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

        let input = CalibrationInput {
            k_slice: &k_slice,
            w_market: &w_market,
            theta_star,
            k_star,
            weights: None,
        };

        let res = calibrate(&input, &NelderMeadConfig::default())
            .expect("calibration must succeed");

        assert!(res.optimizer.converged);
        assert!(res.optimizer.f < 1e-18, "residual: {}", res.optimizer.f);
    }

    #[test]
    fn no_arbitrage_enforced() {
        // With very large eta, calibration should still respect no-arb
        let (k_slice, w_market, theta_star, k_star) = make_sample_slice();
        let input = CalibrationInput {
            k_slice: &k_slice,
            w_market: &w_market,
            theta_star,
            k_star,
            weights: None,
        };

        let res = calibrate(&input, &NelderMeadConfig::default())
            .expect("calibration must succeed");

        assert!(
            ssvi::no_arbitrage_satisfied(res.eta, res.rho),
            "no-arb violated: eta={}, rho={}",
            res.eta,
            res.rho
        );
    }
}
