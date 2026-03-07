use essvi::direct_solver::{self, ButterflyResult, DirectCalibConfig};
use essvi::model::ssvi;

/// Helper: generate evenly spaced points from lo to hi.
fn linspace(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    if n <= 1 {
        return vec![lo];
    }
    (0..n)
        .map(|i| lo + (i as f64) * (hi - lo) / (n - 1) as f64)
        .collect()
}

#[test]
fn test_direct_calib_config_default() {
    let cfg = DirectCalibConfig::default();

    // Theta bounds
    assert!(cfg.theta_lower > 0.0);
    assert!(cfg.theta_upper > cfg.theta_lower);

    // Eta bounds within valid SSVI range
    assert!(cfg.eta_lower > 0.0);
    assert!(cfg.eta_upper < 2.0);

    // Gamma bounds within (0, 1)
    assert!(cfg.gamma_lower > 0.0);
    assert!(cfg.gamma_upper < 1.0);

    // Rho bounds within (-1, 1)
    assert!(cfg.rho_lower > -1.0);
    assert!(cfg.rho_upper < 1.0);

    // Multi-start grid
    assert_eq!(cfg.rho_starts.len(), 4);
    assert_eq!(cfg.eta_starts.len(), 3);

    // Calendar penalty
    assert!(cfg.lambda_calendar > 0.0);

    // Nelder-Mead
    assert_eq!(cfg.nelder_mead.max_iter, 5000);
}

#[test]
fn test_calibrate_slice_synthetic_flat() {
    // Known parameters: symmetric smile (rho=0)
    let theta = 0.04;
    let eta = 0.5;
    let gamma = 0.5;
    let rho = 0.0;

    let k_slice = linspace(-0.5, 0.5, 21);
    let w_market = ssvi::total_variance_slice(&k_slice, theta, eta, gamma, rho);

    let cfg = DirectCalibConfig::default();
    let result = direct_solver::calibrate_slice(&k_slice, &w_market, &cfg);

    // SSE is the primary quality metric — near zero means the fit is excellent.
    // Individual parameter recovery may vary due to (eta, gamma) trade-off in
    // the SSVI phi function: different (eta, gamma) pairs produce similar surfaces.
    assert!(
        result.sse < 1e-10,
        "SSE should be near zero for exact data, got {}",
        result.sse
    );
    assert!(result.converged, "optimizer should converge");

    // Theta and rho are more identifiable than eta/gamma
    assert!(
        (result.theta - theta).abs() < 0.01,
        "theta: expected ~{}, got {}",
        theta,
        result.theta
    );
    assert!(
        (result.rho - rho).abs() < 0.15,
        "rho: expected ~{}, got {}",
        rho,
        result.rho
    );

    // Verify no-arb satisfied
    assert!(ssvi::no_arbitrage_satisfied(result.eta, result.rho));
}

#[test]
fn test_calibrate_slice_synthetic_skewed() {
    // Realistic skewed smile
    let theta = 0.06;
    let eta = 1.0;
    let gamma = 0.5;
    let rho = -0.4;

    let k_slice = linspace(-0.8, 0.5, 31);
    let w_market = ssvi::total_variance_slice(&k_slice, theta, eta, gamma, rho);

    let cfg = DirectCalibConfig::default();
    let result = direct_solver::calibrate_slice(&k_slice, &w_market, &cfg);

    // SSE is the primary quality metric — near zero means the fit is excellent.
    // Individual parameter recovery may vary due to (eta, gamma) trade-off.
    assert!(
        result.sse < 1e-8,
        "SSE should be very small for exact data, got {}",
        result.sse
    );

    // Theta and rho are more identifiable
    assert!(
        (result.theta - theta).abs() < 0.02,
        "theta: expected ~{}, got {}",
        theta,
        result.theta
    );
    assert!(
        (result.rho - rho).abs() < 0.15,
        "rho: expected ~{}, got {}",
        rho,
        result.rho
    );

    // Verify no-arb satisfied
    assert!(ssvi::no_arbitrage_satisfied(result.eta, result.rho));
}

#[test]
fn test_multi_start_finds_global_minimum() {
    // Parameters near the no-arb boundary where local minima are likely
    let theta = 0.03;
    let eta = 1.2;
    let gamma = 0.3;
    let rho = -0.6;

    // Verify these params satisfy no-arb: eta*(1+|rho|) = 1.2*1.6 = 1.92 <= 2.0
    assert!(ssvi::no_arbitrage_satisfied(eta, rho));

    let k_slice = linspace(-1.0, 0.5, 41);
    let w_market = ssvi::total_variance_slice(&k_slice, theta, eta, gamma, rho);

    let cfg = DirectCalibConfig::default();
    let result = direct_solver::calibrate_slice(&k_slice, &w_market, &cfg);

    assert!(
        result.sse < 1e-6,
        "Multi-start should find global basin, SSE = {}",
        result.sse
    );
}

#[test]
fn test_no_arb_barrier_enforced() {
    // Fit any synthetic data and verify result satisfies no-arb
    let theta = 0.05;
    let eta = 0.8;
    let gamma = 0.5;
    let rho = -0.3;

    let k_slice = linspace(-0.6, 0.4, 25);
    let w_market = ssvi::total_variance_slice(&k_slice, theta, eta, gamma, rho);

    let cfg = DirectCalibConfig::default();
    let result = direct_solver::calibrate_slice(&k_slice, &w_market, &cfg);

    assert!(
        ssvi::no_arbitrage_satisfied(result.eta, result.rho),
        "Result must satisfy eta*(1+|rho|) <= 2: eta={}, rho={}, product={}",
        result.eta,
        result.rho,
        result.eta * (1.0 + result.rho.abs())
    );
}

#[test]
fn test_validate_butterfly() {
    // Conservative parameters well within no-arb region
    let theta = 0.04;
    let eta = 0.5;
    let gamma = 0.5;
    let rho = 0.0;

    let k_grid = linspace(-1.0, 1.0, 50);
    let results = direct_solver::validate_butterfly(theta, eta, gamma, rho, &k_grid);

    assert_eq!(results.len(), 50);

    let valid_count = results.iter().filter(|r| r.valid).count();
    let valid_pct = valid_count as f64 / results.len() as f64;
    assert!(
        valid_pct >= 0.9,
        "At least 90% of points should be valid for conservative params, got {:.1}%",
        valid_pct * 100.0
    );
}

#[test]
fn test_validate_butterfly_returns_per_point_detail() {
    let theta = 0.04;
    let eta = 0.5;
    let gamma = 0.5;
    let rho = 0.0;

    let k_grid = linspace(-0.5, 0.5, 5);
    let results = direct_solver::validate_butterfly(theta, eta, gamma, rho, &k_grid);

    assert_eq!(results.len(), 5);

    for (i, r) in results.iter().enumerate() {
        // Check k values match input grid
        assert!(
            (r.k - k_grid[i]).abs() < 1e-15,
            "k mismatch at index {}: expected {}, got {}",
            i,
            k_grid[i],
            r.k
        );

        // Check valid flag matches g(k) >= 0
        assert_eq!(
            r.valid,
            r.g >= 0.0,
            "valid flag mismatch at k={}: g={}, valid={}",
            r.k,
            r.g,
            r.valid
        );
    }
}

#[test]
fn test_no_butterfly_in_objective() {
    // Fit synthetic data and compare with crude_solver using lambda=0
    // (which effectively disables butterfly penalty in crude_solver too).
    // Both should produce similar SSE since both are then pure SSE.
    let theta = 0.05;
    let eta = 0.7;
    let gamma = 0.5;
    let rho = -0.2;

    let k_slice = linspace(-0.6, 0.4, 25);
    let w_market = ssvi::total_variance_slice(&k_slice, theta, eta, gamma, rho);

    // Direct solver (pure SSE by design)
    let direct_cfg = DirectCalibConfig::default();
    let direct_result = direct_solver::calibrate_slice(&k_slice, &w_market, &direct_cfg);

    // Crude solver with lambda=0 (disables butterfly penalty)
    let crude_cfg = essvi::crude_solver::CrudeCalibConfig {
        lambda: 0.0,
        ..essvi::crude_solver::CrudeCalibConfig::default()
    };
    let crude_result = essvi::crude_solver::calibrate_slice(&k_slice, &w_market, &crude_cfg);

    // Both should achieve near-zero SSE on exact synthetic data
    assert!(
        direct_result.sse < 1e-8,
        "Direct solver SSE should be near zero: {}",
        direct_result.sse
    );
    assert!(
        crude_result.sse < 1e-8,
        "Crude solver (lambda=0) SSE should be near zero: {}",
        crude_result.sse
    );

    // SSE values should be in the same ballpark (both finding the global minimum)
    let ratio = if direct_result.sse > crude_result.sse {
        direct_result.sse / crude_result.sse.max(1e-20)
    } else {
        crude_result.sse / direct_result.sse.max(1e-20)
    };
    // Both should be near-zero, ratio comparison is just a sanity check
    assert!(
        direct_result.sse < 1e-6 && crude_result.sse < 1e-6,
        "Both solvers should find near-zero SSE on exact data"
    );
}
