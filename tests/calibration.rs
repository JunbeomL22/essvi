/// SSVI calibration unit tests (migrated from src/calibration.rs).

use essvi::calibration::{
    calibrate, solve_theta, CalibrationConfig, CalibrationInput,
};
use essvi::model::ssvi;

/// Generate a synthetic 20-point smile slice from known SSVI parameters,
/// then calibrate back and verify recovery.
fn make_sample_slice() -> (Vec<f64>, Vec<f64>, f64, f64) {
    let true_eta = 0.8;
    let true_gamma = 0.4;
    let true_rho = -0.35;
    let theta_star = 0.04; // ATM total variance (sigma^2*T ~ 20%^2 * 1Y)
    let k_star = -0.01; // Slightly off-ATM to break eta/gamma degeneracy

    // Solve true theta
    let config = CalibrationConfig::default();
    let true_theta = solve_theta(true_eta, true_gamma, true_rho, theta_star, k_star, &config)
        .expect("true theta must solve");

    // 20 log-moneyness points from -0.5 to +0.5
    let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
    let w_market =
        ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

    (k_slice, w_market, theta_star, k_star)
}

#[test]
fn solve_theta_basic() {
    let config = CalibrationConfig::default();
    let theta = solve_theta(0.8, 0.4, -0.35, 0.04, 0.0, &config);
    assert!(theta.is_ok());
    let t = theta.unwrap();
    // With k*=0, the equation simplifies to theta = theta*
    assert!((t - 0.04).abs() < 1e-10);
}

#[test]
fn solve_theta_nonzero_kstar() {
    let config = CalibrationConfig::default();
    let theta = solve_theta(0.8, 0.4, -0.35, 0.04, 0.01, &config);
    assert!(theta.is_ok());
    let t = theta.unwrap();
    // Verify w(k*, theta) = theta*
    let w = ssvi::total_variance(0.01, t, 0.8, 0.4, -0.35);
    assert!(
        (w - 0.04).abs() < 1e-12,
        "w={}, theta*=0.04, diff={}",
        w,
        (w - 0.04).abs()
    );
}

#[test]
fn calibrate_recovers_parameters() {
    let (k_slice, w_market, theta_star, k_star) = make_sample_slice();

    let config = CalibrationConfig::default();
    let true_theta = solve_theta(0.8, 0.4, -0.35, theta_star, k_star, &config).unwrap();
    let true_phi = ssvi::phi(true_theta, 0.8, 0.4);

    let input = CalibrationInput {
        k_slice: &k_slice,
        w_market: &w_market,
        theta_star,
        k_star,
        weights: None,
    };

    let res = calibrate(&input, &config).expect("calibration must succeed");

    assert!(res.optimizer.converged, "optimizer did not converge");
    assert!(
        res.optimizer.f < 1e-20,
        "residual too large: {}",
        res.optimizer.f
    );

    // rho is directly identifiable from skew shape
    assert!((res.rho - (-0.35)).abs() < 1e-3, "rho: {}", res.rho);

    // phi(theta) is identifiable (controls smile curvature), though eta/gamma individually are not
    let recovered_phi = ssvi::phi(res.theta, res.eta, res.gamma);
    assert!(
        (recovered_phi - true_phi).abs() / true_phi < 1e-3,
        "phi: {} vs true {}",
        recovered_phi,
        true_phi
    );

    // Verify the model reproduces market prices
    let w_fit = ssvi::total_variance_slice(&k_slice, res.theta, res.eta, res.gamma, res.rho);
    let max_err: f64 = w_fit
        .iter()
        .zip(w_market.iter())
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

    let config = CalibrationConfig::default();
    let true_theta = solve_theta(true_eta, true_gamma, true_rho, theta_star, k_star, &config)
        .expect("true theta must solve");

    let k_slice: Vec<f64> = (0..20).map(|i| -0.4 + (i as f64) * 0.04).collect();
    let w_market =
        ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

    let input = CalibrationInput {
        k_slice: &k_slice,
        w_market: &w_market,
        theta_star,
        k_star,
        weights: None,
    };

    let res = calibrate(&input, &config).expect("calibration must succeed");

    assert!(res.optimizer.converged);
    assert!(
        res.optimizer.f < 1e-18,
        "residual: {}",
        res.optimizer.f
    );
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

    let res = calibrate(&input, &CalibrationConfig::default()).expect("calibration must succeed");

    assert!(
        ssvi::no_arbitrage_satisfied(res.eta, res.rho),
        "no-arb violated: eta={}, rho={}",
        res.eta,
        res.rho
    );
}
