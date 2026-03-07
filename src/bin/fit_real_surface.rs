/// Fit SSVI surface to real market data with calendar arbitrage penalty.
///
/// Step 1: Fit each slice independently (no penalty).
/// Step 2: Re-fit sequentially with calendar penalty using previous slice,
///         sampling k from -0.5 to 0.5 step 0.05, lambda=100.

use essvi::calibration::{
    calibrate, calibrate_with_calendar_penalty, CalibrationConfig, CalibrationInput,
    CalibrationResult, PrevSlice,
};
use essvi::fit_common::{build_market_slices, plot_fit, FitResult, SliceData};
use essvi::ssvi;
use std::fs;
use std::io::Write;

fn compute_fit_result(
    slice: &SliceData,
    res: &CalibrationResult,
    prev: Option<&PrevSlice>,
    k_penalty: &[f64],
) -> FitResult {
    let t = slice.t_expiry;
    let w_fit = ssvi::total_variance_slice(&slice.k, res.theta, res.eta, res.gamma, res.rho);
    let iv_fit: Vec<f64> = w_fit.iter().map(|&w| (w / t).max(0.0).sqrt()).collect();

    let iv_errors: Vec<f64> = iv_fit
        .iter()
        .zip(slice.iv.iter())
        .map(|(f, m)| (f - m).abs())
        .collect();
    let max_iv_err = iv_errors.iter().cloned().fold(0.0_f64, f64::max);
    let rmse_iv = (iv_errors.iter().map(|e| e * e).sum::<f64>() / iv_errors.len() as f64).sqrt();
    let avg_price_err = iv_errors.iter().sum::<f64>() / iv_errors.len() as f64;

    // Check calendar violations at penalty points
    let (calendar_violations, max_calendar_violation_bps) = if let Some(prev) = prev {
        let mut violations = 0usize;
        let mut max_viol = 0.0_f64;
        for &k in k_penalty {
            let w_prev = ssvi::total_variance(k, prev.theta, prev.eta, prev.gamma, prev.rho);
            let w_cur = ssvi::total_variance(k, res.theta, res.eta, res.gamma, res.rho);
            if w_prev > w_cur + 1e-14 {
                violations += 1;
                // Convert total variance violation to IV bps (approx)
                let iv_prev = (w_prev / t).max(0.0).sqrt();
                let iv_cur = (w_cur / t).max(0.0).sqrt();
                let viol_bps = (iv_prev - iv_cur).abs() * 10000.0;
                max_viol = max_viol.max(viol_bps);
            }
        }
        (violations, max_viol)
    } else {
        (0, 0.0)
    };

    FitResult {
        t_expiry: t,
        eta: res.eta,
        gamma: res.gamma,
        rho: res.rho,
        theta: res.theta,
        phi: res.phi(),
        no_arb_usage: res.no_arb_usage(),
        converged: res.optimizer.converged,
        max_iv_err_bps: max_iv_err * 10000.0,
        rmse_iv_bps: rmse_iv * 10000.0,
        avg_price_err_bps: avg_price_err * 10000.0,
        calendar_violations,
        max_calendar_violation_bps,
        k: slice.k.clone(),
        iv_market: slice.iv.clone(),
        iv_fit,
    }
}

// ── Main ────────────────────────────────────────────────────

fn main() {
    let plot_dir = "documents/plots";
    fs::create_dir_all(plot_dir).expect("create plot dir");

    let slices = build_market_slices();
    let config = CalibrationConfig::default();

    // Penalty sample points: k from -0.5 to 0.5 step 0.05
    let k_penalty: Vec<f64> = (0..=20).map(|i| -0.5 + i as f64 * 0.05).collect();
    let lambda = 100.0;

    // ── Step 1: Unconstrained per-slice fit ──────────────────
    println!("=== Step 1: Unconstrained per-slice fit ===");
    let mut unconstrained: Vec<(CalibrationResult, FitResult)> = Vec::new();

    for slice in &slices {
        let t = slice.t_expiry;
        let w_market: Vec<f64> = slice.iv.iter().map(|&s| s * s * t).collect();
        let atm_vol = slice.iv[slice.k.len() / 2];
        let theta_star = atm_vol * atm_vol * t;
        let k_star = slice.k[slice.k.len() / 2];

        let weights: Vec<f64> = slice.k.iter().map(|&k| {
            if k >= -0.2 && k <= 0.2 { 3.0 } else { 1.0 }
        }).collect();

        let input = CalibrationInput {
            k_slice: &slice.k,
            w_market: &w_market,
            theta_star,
            k_star,
            weights: Some(&weights),
        };

        match calibrate(&input, &config) {
            Ok(res) => {
                let prev = if unconstrained.is_empty() {
                    None
                } else {
                    let p = &unconstrained.last().unwrap().0;
                    Some(PrevSlice {
                        theta: p.theta,
                        eta: p.eta,
                        gamma: p.gamma,
                        rho: p.rho,
                    })
                };
                let fr = compute_fit_result(slice, &res, prev.as_ref(), &k_penalty);
                println!(
                    "T={:.4}: max_err={:.1} bps, RMSE={:.1} bps, cal_viol={}",
                    t, fr.max_iv_err_bps, fr.rmse_iv_bps, fr.calendar_violations
                );
                unconstrained.push((res, fr));
            }
            Err(e) => {
                eprintln!("Calibration FAILED for T={}: {}", t, e);
            }
        }
    }

    // ── Step 2: Sequential refit with calendar penalty ───────
    println!("\n=== Step 2: Surface fit with calendar penalty (lambda={}) ===", lambda);
    let mut surface_results: Vec<FitResult> = Vec::new();

    for (i, slice) in slices.iter().enumerate() {
        let t = slice.t_expiry;
        let w_market: Vec<f64> = slice.iv.iter().map(|&s| s * s * t).collect();
        let atm_vol = slice.iv[slice.k.len() / 2];
        let theta_star = atm_vol * atm_vol * t;
        let k_star = slice.k[slice.k.len() / 2];

        let weights: Vec<f64> = slice.k.iter().map(|&k| {
            if k >= -0.2 && k <= 0.2 { 3.0 } else { 1.0 }
        }).collect();

        let input = CalibrationInput {
            k_slice: &slice.k,
            w_market: &w_market,
            theta_star,
            k_star,
            weights: Some(&weights),
        };

        // First slice: no penalty (no previous slice)
        if i == 0 {
            let res = calibrate(&input, &config).expect("first slice must calibrate");
            let fr = compute_fit_result(slice, &res, None, &k_penalty);
            println!(
                "T={:.4}: max_err={:.1} bps, RMSE={:.1} bps, cal_viol={}, (no prev)",
                t, fr.max_iv_err_bps, fr.rmse_iv_bps, fr.calendar_violations
            );
            surface_results.push(fr);
        } else {
            // Use unconstrained fit as initial guess
            let unc = &unconstrained[i].0;
            let init = [unc.eta, unc.gamma, unc.rho];

            // Previous slice from surface results
            let prev_fr = &surface_results[i - 1];
            let prev = PrevSlice {
                theta: prev_fr.theta,
                eta: prev_fr.eta,
                gamma: prev_fr.gamma,
                rho: prev_fr.rho,
            };

            let res = calibrate_with_calendar_penalty(&input, &config, &prev, &k_penalty, lambda, &init)
                .expect("surface calibration must succeed");
            let fr = compute_fit_result(slice, &res, Some(&prev), &k_penalty);
            println!(
                "T={:.4}: max_err={:.1} bps, RMSE={:.1} bps, cal_viol={}, max_cal_viol={:.1} bps",
                t, fr.max_iv_err_bps, fr.rmse_iv_bps, fr.calendar_violations, fr.max_calendar_violation_bps
            );
            surface_results.push(fr);
        }
    }

    // ── Generate plots ──────────────────────────────────────
    for r in &surface_results {
        let t_str = format!("{:.3}", r.t_expiry).replace('.', "p");
        let path = format!("{}/fit_surface_T{}.svg", plot_dir, t_str);
        let title = format!(
            "T={:.4} (surface fit) avg price err: {:.1} bps, cal viol: {}",
            r.t_expiry, r.avg_price_err_bps, r.calendar_violations
        );
        if let Err(e) = plot_fit(&r, &path, &title) {
            eprintln!("Plot error for T={}: {}", r.t_expiry, e);
        }
    }

    // ── Write markdown report ───────────────────────────────
    let mut md = String::new();

    md.push_str("# SSVI Real-World Surface Fit Report\n\n");
    md.push_str("Surface calibration with calendar arbitrage penalty.\n\n");
    md.push_str("- **Step 1**: Unconstrained per-slice fit (3x weight for k in [-0.2, 0.2])\n");
    md.push_str(&format!("- **Step 2**: Sequential refit with calendar penalty (lambda={}, k penalty points: -0.5 to 0.5, step 0.05)\n\n", lambda));

    // Unconstrained summary
    md.push_str("## Step 1: Unconstrained Fit\n\n");
    md.push_str("| T | max IV err (bps) | RMSE IV (bps) | eta | gamma | rho | cal violations |\n");
    md.push_str("|------:|-----------------:|--------------:|------:|------:|------:|--------------:|\n");
    for (_, fr) in &unconstrained {
        md.push_str(&format!(
            "| {:.4} | {:.1} | {:.1} | {:.4} | {:.4} | {:.4} | {} |\n",
            fr.t_expiry, fr.max_iv_err_bps, fr.rmse_iv_bps, fr.eta, fr.gamma, fr.rho, fr.calendar_violations
        ));
    }
    md.push_str("\n");

    // Surface fit summary
    md.push_str("## Step 2: Surface Fit (with calendar penalty)\n\n");
    md.push_str("| T | max IV err (bps) | RMSE IV (bps) | avg price err (bps) | eta | gamma | rho | phi | eta*(1+\\|rho\\|) | cal violations | max cal viol (bps) | converged |\n");
    md.push_str("|------:|-----------------:|--------------:|--------------------:|------:|------:|------:|------:|---------------:|--------------:|------------------:|:---------:|\n");
    for r in &surface_results {
        md.push_str(&format!(
            "| {:.4} | {:.1} | {:.1} | {:.1} | {:.4} | {:.4} | {:.4} | {:.3} | {:.3} | {} | {:.1} | {} |\n",
            r.t_expiry,
            r.max_iv_err_bps,
            r.rmse_iv_bps,
            r.avg_price_err_bps,
            r.eta,
            r.gamma,
            r.rho,
            r.phi,
            r.no_arb_usage,
            r.calendar_violations,
            r.max_calendar_violation_bps,
            if r.converged { "yes" } else { "**no**" }
        ));
    }
    md.push_str("\n");

    // Fit plots
    md.push_str("## Fit Plots\n\n");
    for r in &surface_results {
        let t_str = format!("{:.3}", r.t_expiry).replace('.', "p");
        md.push_str(&format!("### T = {:.4}\n\n", r.t_expiry));
        md.push_str(&format!(
            "max err: {:.1} bps | RMSE: {:.1} bps | eta={:.4}, gamma={:.4}, rho={:.4} | cal violations: {}\n\n",
            r.max_iv_err_bps, r.rmse_iv_bps, r.eta, r.gamma, r.rho, r.calendar_violations
        ));
        md.push_str(&format!("![T={:.4}](plots/fit_surface_T{}.svg)\n\n", r.t_expiry, t_str));
    }

    // Calendar arbitrage analysis
    md.push_str("## Calendar Arbitrage Analysis\n\n");
    md.push_str("Violations checked at 21 k-points from -0.5 to 0.5 (step 0.05).\n\n");
    md.push_str("| T | cal violations | max cal viol (bps) | eta*(1+\\|rho\\|) |\n");
    md.push_str("|------:|--------------:|------------------:|---------------:|\n");
    for r in &surface_results {
        md.push_str(&format!(
            "| {:.4} | {} | {:.1} | {:.3} |\n",
            r.t_expiry, r.calendar_violations, r.max_calendar_violation_bps, r.no_arb_usage
        ));
    }

    let report_path = "documents/real-world-surface-fit.md";
    let mut file = fs::File::create(report_path).expect("create report file");
    file.write_all(md.as_bytes()).expect("write report");

    println!("\nReport written to {}", report_path);
    println!("Plots written to {}/", plot_dir);
}
