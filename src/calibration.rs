/// SSVI calibration: solve θ implicitly, then minimize ||W - tv||².

use crate::brent::brent;
use crate::nelder_mead::{nelder_mead_bounded, NelderMeadConfig, NelderMeadResult};
use crate::ssvi;

/// Solve θ from the implicit ATM consistency equation:
///   θ = θ* / (1 + ρ · φ(θ) · k*)
/// using Brent's method on h(θ) = θ - θ*/(1 + ρ·φ(θ)·k*) = 0.
pub fn solve_theta(
    eta: f64,
    gamma: f64,
    rho: f64,
    theta_star: f64,
    k_star: f64,
) -> Option<f64> {
    // When k_star=0, the equation reduces to θ = θ* (no φ dependence).
    if k_star.abs() < 1e-15 {
        return Some(theta_star);
    }

    let h = |theta: f64| -> f64 {
        if theta <= 0.0 {
            return -theta_star;
        }
        let p = ssvi::phi(theta, eta, gamma);
        theta - theta_star / (1.0 + rho * p * k_star)
    };

    // Bracket: root is near theta_star, search [theta_star/1000, 100*theta_star]
    let lo = theta_star * 1e-3;
    let hi = theta_star * 100.0;
    let res = brent(h, lo, hi, 1e-14, 200);
    if res.converged && res.root > 0.0 {
        Some(res.root)
    } else {
        None
    }
}

/// Squared error between model and market total variances.
fn squared_error(model: &[f64], market: &[f64]) -> f64 {
    model
        .iter()
        .zip(market.iter())
        .map(|(m, w)| (m - w).powi(2))
        .sum()
}

#[derive(Debug, Clone)]
pub struct CalibrationInput<'a> {
    pub k_slice: &'a [f64],
    pub w_market: &'a [f64],
    pub theta_star: f64,
    pub k_star: f64,
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
            squared_error(&w_model, input.w_market)
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
        squared_error(&w_model, input.w_market)
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
        // Verify the implicit equation holds
        let p = crate::ssvi::phi(t, 0.8, 0.4);
        let rhs = 0.04 / (1.0 + (-0.35) * p * 0.01);
        assert!((t - rhs).abs() < 1e-12);
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
