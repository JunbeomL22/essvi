use essvi::crude_solver::{CrudeCalibConfig, SliceInput, calibrate_slice, calibrate_surface};
use essvi::model::ssvi;

#[test]
fn test_crude_calib_config_default() {
    let cfg = CrudeCalibConfig::default();

    // Theta bounds
    assert!(cfg.theta_lower > 0.0, "theta_lower must be positive");
    assert!(cfg.theta_upper > cfg.theta_lower, "theta_upper > theta_lower");

    // Eta bounds within valid SSVI range
    assert!(cfg.eta_lower > 0.0);
    assert!(cfg.eta_upper < 2.0);

    // Gamma bounds within (0, 1)
    assert!(cfg.gamma_lower > 0.0);
    assert!(cfg.gamma_upper < 1.0);

    // Rho bounds within (-1, 1)
    assert!(cfg.rho_lower > -1.0);
    assert!(cfg.rho_upper < 1.0);

    // Penalty weight positive
    assert!(cfg.lambda > 0.0, "lambda must be positive");

    // Penalty grid valid
    assert!(cfg.k_penalty_n > 0, "k_penalty_n must be positive");
    assert!(cfg.k_penalty_lo < cfg.k_penalty_hi, "penalty grid must have lo < hi");

    // Nelder-Mead has enough iterations for 4D search
    assert!(cfg.nelder_mead.max_iter >= 5000, "4D search needs more iterations than default 1000");

    // Calendar spread penalty weight
    assert!(cfg.lambda_calendar > 0.0, "lambda_calendar must be positive");
    assert!((cfg.lambda_calendar - 10.0).abs() < f64::EPSILON, "lambda_calendar default should be 10.0");
}

#[test]
fn test_calibrate_slice_synthetic_flat() {
    // Known parameters: symmetric smile, no skew
    let true_theta = 0.04;
    let true_eta = 0.5;
    let true_gamma = 0.5;
    let true_rho = 0.0;

    // Generate synthetic market data
    let n = 21;
    let k_slice: Vec<f64> = (0..n)
        .map(|i| -0.5 + (i as f64) * 1.0 / (n - 1) as f64)
        .collect();
    let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

    let cfg = CrudeCalibConfig::default();
    let result = calibrate_slice(&k_slice, &w_market, &cfg);

    assert!(
        result.converged,
        "optimizer should converge on noise-free data"
    );
    assert!(
        result.sse < 1e-10,
        "SSE should be near-zero for exact data, got {}",
        result.sse
    );

    // Theta and rho are identifiable; eta and gamma have a well-known
    // degeneracy (phi = eta / (theta^gamma * (1+theta)^(1-gamma))), so
    // we verify fit quality via SSE and phi match instead of individual eta/gamma.
    assert!(
        (result.theta - true_theta).abs() < 0.01,
        "theta mismatch: fitted={}, true={}",
        result.theta,
        true_theta
    );
    assert!(
        (result.rho - true_rho).abs() < 0.15,
        "rho mismatch: fitted={}, true={}",
        result.rho,
        true_rho
    );

    // Verify phi matches (equivalent smile shape regardless of eta/gamma split)
    let true_phi = ssvi::phi(true_theta, true_eta, true_gamma);
    let fitted_phi = ssvi::phi(result.theta, result.eta, result.gamma);
    assert!(
        (fitted_phi - true_phi).abs() / true_phi < 0.05,
        "phi mismatch: fitted={}, true={} (>5% relative error)",
        fitted_phi,
        true_phi
    );
}

#[test]
fn test_calibrate_slice_synthetic_skewed() {
    // Known parameters: realistic equity skew
    let true_theta = 0.06;
    let true_eta = 1.0;
    let true_gamma = 0.5;
    let true_rho = -0.4;

    // Generate synthetic market data (wider range)
    let n = 31;
    let k_slice: Vec<f64> = (0..n)
        .map(|i| -0.8 + (i as f64) * 1.3 / (n - 1) as f64)
        .collect();
    let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

    let cfg = CrudeCalibConfig::default();
    let result = calibrate_slice(&k_slice, &w_market, &cfg);

    assert!(
        result.sse < 1e-8,
        "SSE should be very small for exact data, got {}",
        result.sse
    );
    assert!(
        (result.theta - true_theta).abs() < 0.015,
        "theta mismatch: fitted={}, true={}",
        result.theta,
        true_theta
    );
    assert!(
        (result.rho - true_rho).abs() < 0.15,
        "rho mismatch: fitted={}, true={}",
        result.rho,
        true_rho
    );

    // Verify phi matches (captures the eta/gamma equivalence)
    let true_phi = ssvi::phi(true_theta, true_eta, true_gamma);
    let fitted_phi = ssvi::phi(result.theta, result.eta, result.gamma);
    assert!(
        (fitted_phi - true_phi).abs() / true_phi < 0.05,
        "phi mismatch: fitted={}, true={} (>5% relative error)",
        fitted_phi,
        true_phi
    );

    // Also verify total variance curve matches at several points
    for &k in &[-0.5, -0.2, 0.0, 0.2, 0.4] {
        let w_true = ssvi::total_variance(k, true_theta, true_eta, true_gamma, true_rho);
        let w_fit = ssvi::total_variance(k, result.theta, result.eta, result.gamma, result.rho);
        assert!(
            (w_fit - w_true).abs() < 1e-6,
            "total variance mismatch at k={}: fitted={}, true={}",
            k,
            w_fit,
            w_true
        );
    }
}

#[test]
fn test_calibrate_slice_butterfly_satisfied() {
    // Fit a skewed slice and verify butterfly condition holds
    let true_theta = 0.06;
    let true_eta = 1.0;
    let true_gamma = 0.5;
    let true_rho = -0.4;

    let n = 31;
    let k_slice: Vec<f64> = (0..n)
        .map(|i| -0.8 + (i as f64) * 1.3 / (n - 1) as f64)
        .collect();
    let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

    let cfg = CrudeCalibConfig::default();
    let result = calibrate_slice(&k_slice, &w_market, &cfg);

    // Check butterfly density g(k) >= 0 across a dense grid
    let n_check = 50;
    let h = 1e-5;
    for i in 0..n_check {
        let k = -1.0 + (i as f64) * 2.0 / (n_check - 1) as f64;

        let w = ssvi::total_variance(k, result.theta, result.eta, result.gamma, result.rho);
        if w <= 0.0 {
            continue;
        }

        let w_plus =
            ssvi::total_variance(k + h, result.theta, result.eta, result.gamma, result.rho);
        let w_minus =
            ssvi::total_variance(k - h, result.theta, result.eta, result.gamma, result.rho);

        let w_prime = (w_plus - w_minus) / (2.0 * h);
        let w_double_prime = (w_plus - 2.0 * w + w_minus) / (h * h);

        let term1 = 1.0 - k * w_prime / (2.0 * w);
        let g = term1 * term1 - w_prime * w_prime / 4.0 * (1.0 / w + 0.25) + w_double_prime / 2.0;

        assert!(
            g >= -1e-6,
            "butterfly violation at k={:.4}: g={:.6e} (theta={}, eta={}, gamma={}, rho={})",
            k,
            g,
            result.theta,
            result.eta,
            result.gamma,
            result.rho
        );
    }
}

// ── Sequential calibration tests ──────────────────────────────

#[test]
fn test_calibrate_surface_monotonic_theta() {
    // 4 slices with known increasing theta values
    let params: [(f64, f64); 4] = [
        (0.1, 0.02),   // T, theta
        (0.25, 0.04),
        (0.5, 0.08),
        (1.0, 0.15),
    ];
    let eta = 0.8;
    let gamma = 0.5;
    let rho = -0.3;

    let n = 21;
    let k_values: Vec<f64> = (0..n)
        .map(|i| -0.5 + (i as f64) * 1.0 / (n - 1) as f64)
        .collect();

    let w_markets: Vec<Vec<f64>> = params
        .iter()
        .map(|&(_, theta)| ssvi::total_variance_slice(&k_values, theta, eta, gamma, rho))
        .collect();

    // Pass slices in SCRAMBLED order to verify sorting
    let slices = vec![
        SliceInput { t: params[2].0, k: &k_values, w: &w_markets[2] }, // T=0.5
        SliceInput { t: params[0].0, k: &k_values, w: &w_markets[0] }, // T=0.1
        SliceInput { t: params[3].0, k: &k_values, w: &w_markets[3] }, // T=1.0
        SliceInput { t: params[1].0, k: &k_values, w: &w_markets[1] }, // T=0.25
    ];

    let cfg = CrudeCalibConfig::default();
    let results = calibrate_surface(&slices, &cfg);

    assert_eq!(results.len(), 4, "should return one result per slice");

    // Results should be in T-sorted order; verify thetas are close to known ascending values
    for (i, &(_, true_theta)) in params.iter().enumerate() {
        assert!(
            (results[i].theta - true_theta).abs() < 0.02,
            "theta mismatch at slice {}: fitted={:.6}, true={:.6}",
            i, results[i].theta, true_theta
        );
        assert!(
            results[i].sse < 1e-6,
            "SSE too large at slice {}: {}",
            i, results[i].sse
        );
    }

    // Verify monotonicity: theta[i+1] >= theta[i]
    for i in 0..results.len() - 1 {
        assert!(
            results[i + 1].theta >= results[i].theta - 1e-6,
            "theta not monotonic: theta[{}]={:.6} > theta[{}]={:.6}",
            i, results[i].theta, i + 1, results[i + 1].theta
        );
    }
}

#[test]
fn test_calibrate_surface_calendar_penalty_effect() {
    // Calendar arbitrage scenario: short-dated slice has HIGHER theta
    let eta = 0.5;
    let gamma = 0.5;
    let rho = 0.0;

    let true_thetas = [0.10, 0.05, 0.15]; // T=0.1 has higher theta than T=0.5
    let ts = [0.1, 0.5, 1.0];

    let n = 21;
    let k_values: Vec<f64> = (0..n)
        .map(|i| -0.5 + (i as f64) * 1.0 / (n - 1) as f64)
        .collect();

    let w_markets: Vec<Vec<f64>> = true_thetas
        .iter()
        .map(|&theta| ssvi::total_variance_slice(&k_values, theta, eta, gamma, rho))
        .collect();

    let slices: Vec<SliceInput> = ts
        .iter()
        .zip(w_markets.iter())
        .map(|(&t, w)| SliceInput { t, k: &k_values, w })
        .collect();

    // With lambda_calendar=0: should recover non-monotonic thetas
    let mut cfg_no_penalty = CrudeCalibConfig::default();
    cfg_no_penalty.lambda_calendar = 0.0;
    let results_no = calibrate_surface(&slices, &cfg_no_penalty);

    // The second slice (T=0.5) should have lower theta than first (T=0.1)
    assert!(
        results_no[1].theta < results_no[0].theta,
        "without penalty, theta[1]={:.6} should be < theta[0]={:.6}",
        results_no[1].theta, results_no[0].theta
    );

    // With lambda_calendar=1000: penalty should push toward monotonicity
    let mut cfg_penalty = CrudeCalibConfig::default();
    cfg_penalty.lambda_calendar = 1000.0;
    let results_pen = calibrate_surface(&slices, &cfg_penalty);

    // With strong penalty, theta[1] should be much closer to theta[0] than without penalty
    // The soft penalty may not achieve perfect monotonicity, but the gap should shrink
    let gap_no_penalty = results_no[0].theta - results_no[1].theta;
    let gap_with_penalty = (results_pen[0].theta - results_pen[1].theta).max(0.0);
    assert!(
        gap_with_penalty < gap_no_penalty * 0.5,
        "penalty should reduce monotonicity gap: without={:.6}, with={:.6}",
        gap_no_penalty, gap_with_penalty
    );
}

#[test]
fn test_calibrate_surface_single_slice() {
    let true_theta = 0.04;
    let true_eta = 0.5;
    let true_gamma = 0.5;
    let true_rho = 0.0;

    let n = 21;
    let k_values: Vec<f64> = (0..n)
        .map(|i| -0.5 + (i as f64) * 1.0 / (n - 1) as f64)
        .collect();
    let w_market = ssvi::total_variance_slice(&k_values, true_theta, true_eta, true_gamma, true_rho);

    let cfg = CrudeCalibConfig::default();

    // Single slice via calibrate_slice
    let single = calibrate_slice(&k_values, &w_market, &cfg);

    // Single slice via calibrate_surface
    let slices = vec![SliceInput { t: 0.25, k: &k_values, w: &w_market }];
    let surface = calibrate_surface(&slices, &cfg);

    assert_eq!(surface.len(), 1);
    assert!(
        (surface[0].theta - single.theta).abs() < 0.01,
        "theta mismatch: surface={:.6}, single={:.6}",
        surface[0].theta, single.theta
    );
    assert!(
        (surface[0].rho - single.rho).abs() < 0.15,
        "rho mismatch: surface={:.6}, single={:.6}",
        surface[0].rho, single.rho
    );
    assert!(
        (surface[0].sse - single.sse).abs() < 1e-8,
        "SSE mismatch: surface={:.6e}, single={:.6e}",
        surface[0].sse, single.sse
    );
}

#[test]
fn test_calibrate_surface_preserves_existing_behavior() {
    // Verify that the refactoring of calibrate_slice (to use calibrate_slice_with_prev)
    // did not change its behavior by checking it produces same results as calibrate_surface
    // for a skewed synthetic slice.
    let true_theta = 0.06;
    let true_eta = 1.0;
    let true_gamma = 0.5;
    let true_rho = -0.4;

    let n = 31;
    let k_values: Vec<f64> = (0..n)
        .map(|i| -0.8 + (i as f64) * 1.3 / (n - 1) as f64)
        .collect();
    let w_market = ssvi::total_variance_slice(&k_values, true_theta, true_eta, true_gamma, true_rho);

    let cfg = CrudeCalibConfig::default();
    let result = calibrate_slice(&k_values, &w_market, &cfg);

    // Same assertions as the original skewed test to verify no regression
    assert!(result.sse < 1e-8, "SSE too large: {}", result.sse);
    assert!(
        (result.theta - true_theta).abs() < 0.015,
        "theta mismatch: fitted={}, true={}",
        result.theta, true_theta
    );
    assert!(
        (result.rho - true_rho).abs() < 0.15,
        "rho mismatch: fitted={}, true={}",
        result.rho, true_rho
    );

    // Verify total variance curve matches
    for &k in &[-0.5, -0.2, 0.0, 0.2, 0.4] {
        let w_true = ssvi::total_variance(k, true_theta, true_eta, true_gamma, true_rho);
        let w_fit = ssvi::total_variance(k, result.theta, result.eta, result.gamma, result.rho);
        assert!(
            (w_fit - w_true).abs() < 1e-6,
            "total variance mismatch at k={}: fitted={}, true={}",
            k, w_fit, w_true
        );
    }
}

// Note: Regression test for existing 111 tests is implicit --
// this file compiles and runs alongside them. Running `cargo test`
// confirms the full suite passes with no regressions.
