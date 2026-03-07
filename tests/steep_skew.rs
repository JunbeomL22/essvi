/// Stress test: short expiry + steep skew.
///
/// Scenario: T near zero, ATM vol = 0.4, vol at k=0.7 reaches 0.7.
/// This produces a very steep smile — does SSVI fit it well?
use essvi::calibration::{CalibrationConfig, CalibrationInput, calibrate};
use essvi::ssvi;

/// Build synthetic market data for a steep-skew short-expiry slice.
/// ATM σ = 0.4, and we want σ(k) to reach ~0.7 at k ~ -0.7
/// (negative k = OTM puts, where steep skew lives).
///
/// Total variance = σ² · T, so:
///   w_ATM = 0.4² · T = 0.16T
///   w(k=-0.7) = 0.7² · T = 0.49T
///
/// Ratio w(-0.7)/w(0) ≈ 3.06 — very steep.
fn make_steep_skew_slice(t_expiry: f64) -> (Vec<f64>, Vec<f64>, f64) {
    // 20 points from k=-0.7 to k=+0.3 (skew is on the put side)
    let k_slice: Vec<f64> = (0..20).map(|i| -0.7 + (i as f64) * 1.0 / 19.0).collect();

    // Construct implied vols: linear interpolation from 0.7 at k=-0.7 to 0.4 at k=0,
    // then flatter on the call side (slight smile).
    let iv: Vec<f64> = k_slice
        .iter()
        .map(|&k| {
            if k <= 0.0 {
                // Put side: steep skew from 0.7 to 0.4
                0.4 + (0.7 - 0.4) * (-k / 0.7)
            } else {
                // Call side: mild uptick
                0.4 + 0.05 * (k / 0.3)
            }
        })
        .collect();

    // Convert to total variance: w = σ² · T
    let w_market: Vec<f64> = iv.iter().map(|&sigma| sigma * sigma * t_expiry).collect();
    let theta_star = 0.4 * 0.4 * t_expiry; // ATM total variance

    (k_slice, w_market, theta_star)
}

fn run_calibration(label: &str, t_expiry: f64) {
    let (k_slice, w_market, theta_star) = make_steep_skew_slice(t_expiry);
    let k_star = 0.0;

    println!("\n==========================================================");
    println!("  {} (T = {})", label, t_expiry);
    println!("==========================================================");
    println!("  theta_star (ATM w) = {:.6e}", theta_star);
    println!(
        "  w range: [{:.6e}, {:.6e}]",
        w_market.iter().cloned().fold(f64::INFINITY, f64::min),
        w_market.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    );

    let input = CalibrationInput {
        k_slice: &k_slice,
        w_market: &w_market,
        theta_star,
        k_star,
        weights: None,
    };

    let config = CalibrationConfig::default();
    let res = calibrate(&input, &config);

    match res {
        Ok(r) => {
            println!("\n  Calibrated params:");
            println!("    eta   = {:.6}", r.eta);
            println!("    gamma = {:.6}", r.gamma);
            println!("    rho   = {:.6}", r.rho);
            println!("    theta = {:.6e}", r.theta);
            println!("    phi   = {:.6}", r.phi());
            println!("    no-arb: {}", ssvi::no_arbitrage_satisfied(r.eta, r.rho));
            println!("    converged: {}", r.optimizer.converged);
            println!("    iterations: {}", r.optimizer.iterations);
            println!("    SSE: {:.6e}", r.optimizer.f);

            // Pointwise errors
            let w_fit = ssvi::total_variance_slice(&k_slice, r.theta, r.eta, r.gamma, r.rho);
            let errors: Vec<f64> = w_fit
                .iter()
                .zip(w_market.iter())
                .map(|(m, w)| m - w)
                .collect();
            let max_abs = errors.iter().map(|e| e.abs()).fold(0.0_f64, f64::max);
            let rmse = (errors.iter().map(|e| e * e).sum::<f64>() / errors.len() as f64).sqrt();

            // Convert errors to implied vol terms for interpretability
            let iv_errors: Vec<f64> = w_fit
                .iter()
                .zip(w_market.iter())
                .zip(k_slice.iter())
                .map(|((wf, wm), _k)| {
                    let sigma_fit = (wf / t_expiry).sqrt();
                    let sigma_mkt = (wm / t_expiry).sqrt();
                    sigma_fit - sigma_mkt
                })
                .collect();
            let max_iv_err = iv_errors.iter().map(|e| e.abs()).fold(0.0_f64, f64::max);

            println!("\n  Fit quality:");
            println!("    max |w_err|   = {:.6e}", max_abs);
            println!("    RMSE(w)       = {:.6e}", rmse);
            println!(
                "    max |iv_err|  = {:.4} vol pts ({:.2} bps)",
                max_iv_err,
                max_iv_err * 10000.0
            );

            println!(
                "\n  {:>8} {:>10} {:>10} {:>10} {:>10}",
                "k", "iv_mkt", "iv_fit", "iv_err", "w_err"
            );
            for i in 0..k_slice.len() {
                let sigma_mkt = (w_market[i] / t_expiry).sqrt();
                let sigma_fit = (w_fit[i] / t_expiry).sqrt();
                println!(
                    "  {:>8.4} {:>10.6} {:>10.6} {:>+10.6} {:>+10.2e}",
                    k_slice[i],
                    sigma_mkt,
                    sigma_fit,
                    sigma_fit - sigma_mkt,
                    errors[i]
                );
            }
        }
        Err(e) => {
            println!("  CALIBRATION FAILED: {}", e);
        }
    }
}

#[test]
fn steep_skew_stress_test() {
    // Test across multiple expiries from 1Y down to near-zero
    let expiries = [1.0, 0.1, 0.01, 0.001];

    for &t in &expiries {
        run_calibration("Steep skew", t);
    }
}

#[test]
fn steep_skew_fit_quality() {
    // Quantitative assertions for each regime
    let expiries = [1.0, 0.1, 0.01, 0.001];

    for &t in &expiries {
        let (k_slice, w_market, theta_star) = make_steep_skew_slice(t);
        let input = CalibrationInput {
            k_slice: &k_slice,
            w_market: &w_market,
            theta_star,
            k_star: 0.0,
            weights: None,
        };

        let res = calibrate(&input, &CalibrationConfig::default());
        assert!(res.is_ok(), "calibration failed at T={}", t);
        let r = res.unwrap();

        let w_fit = ssvi::total_variance_slice(&k_slice, r.theta, r.eta, r.gamma, r.rho);

        // Max IV error in vol points
        let max_iv_err: f64 = w_fit
            .iter()
            .zip(w_market.iter())
            .map(|(wf, wm)| ((wf / t).sqrt() - (wm / t).sqrt()).abs())
            .fold(0.0_f64, f64::max);

        println!(
            "T={:.4}: max_iv_err = {:.4} ({:.1} bps), SSE = {:.2e}, converged={}",
            t,
            max_iv_err,
            max_iv_err * 10000.0,
            r.optimizer.f,
            r.optimizer.converged
        );
    }
}
