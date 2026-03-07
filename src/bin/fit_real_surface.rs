/// Fit SSVI surface to real market data with calendar arbitrage penalty.
///
/// Step 1: Fit each slice independently (no penalty).
/// Step 2: Re-fit sequentially with calendar penalty using previous slice,
///         sampling k from -0.5 to 0.5 step 0.05, lambda=100.

use essvi::calibration::{
    calibrate, calibrate_with_calendar_penalty, CalibrationConfig, CalibrationInput,
    CalibrationResult, PrevSlice,
};
use essvi::ssvi;
use plotters::prelude::*;
use std::fs;
use std::io::Write;

// ── Slice definition (same as fit_real.rs) ──────────────────

struct SliceData {
    t_expiry: f64,
    k: Vec<f64>,
    iv: Vec<f64>,
}

fn make_slice(
    t_expiry: f64,
    k_lo: f64,
    k_hi: f64,
    n: usize,
    atm_vol: f64,
    skew: f64,
    curvature: f64,
    noise_amp: f64,
) -> SliceData {
    let mut k_vals = Vec::with_capacity(n);
    let mut iv_vals = Vec::with_capacity(n);
    let seed = (t_expiry * 10000.0) as u64;
    let rho_shape = -0.5; // less negative = more call-side vol
    let d = 0.08 + 0.05 * t_expiry.sqrt();
    let a = skew * 0.45;

    for i in 0..n {
        let frac = i as f64 / (n - 1) as f64;
        let k = k_lo + frac * (k_hi - k_lo);
        let hyp = a * (rho_shape * k + (k * k + d * d).sqrt());
        let wing = curvature * 0.01 * k * k;
        let hash = ((seed.wrapping_mul(31).wrapping_add(i as u64)).wrapping_mul(7919)) % 1000;
        let noise = noise_amp * ((hash as f64 / 500.0) - 1.0);
        let sigma = (atm_vol + hyp + wing + noise).max(0.02);
        k_vals.push(k);
        iv_vals.push(sigma);
    }

    SliceData {
        t_expiry,
        k: k_vals,
        iv: iv_vals,
    }
}

fn build_market_slices() -> Vec<SliceData> {
    vec![
        make_slice(0.0301, -0.30, 0.20, 60, 0.200, 0.28, 0.08, 0.0015),
        make_slice(0.1068, -0.40, 0.25, 60, 0.185, 0.24, 0.06, 0.0012),
        make_slice(0.1936, -0.45, 0.30, 60, 0.175, 0.22, 0.06, 0.0010),
        make_slice(0.2795, -0.50, 0.35, 60, 0.170, 0.20, 0.05, 0.0010),
        make_slice(0.4376, -0.55, 0.40, 60, 0.165, 0.18, 0.04, 0.0008),
        make_slice(0.7014, -0.55, 0.45, 60, 0.158, 0.15, 0.04, 0.0008),
        make_slice(0.9507, -0.55, 0.45, 60, 0.155, 0.14, 0.04, 0.0007),
        make_slice(1.0274, -0.55, 0.45, 60, 0.153, 0.13, 0.04, 0.0007),
        make_slice(1.1988, -0.60, 0.50, 60, 0.152, 0.12, 0.035, 0.0007),
        make_slice(1.4495, -0.65, 0.55, 60, 0.155, 0.11, 0.03, 0.0006),
        make_slice(1.9476, -0.75, 0.60, 60, 0.160, 0.10, 0.025, 0.0006),
        make_slice(2.9452, -0.90, 0.65, 60, 0.168, 0.09, 0.02, 0.0005),
    ]
}

// ── Fit result ──────────────────────────────────────────────

struct FitResult {
    t_expiry: f64,
    eta: f64,
    gamma: f64,
    rho: f64,
    theta: f64,
    phi: f64,
    no_arb_usage: f64,
    converged: bool,
    max_iv_err_bps: f64,
    rmse_iv_bps: f64,
    avg_price_err_bps: f64,
    calendar_violations: usize,
    max_calendar_violation_bps: f64,
    k: Vec<f64>,
    iv_market: Vec<f64>,
    iv_fit: Vec<f64>,
}

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

// ── Plot ────────────────────────────────────────────────────

fn plot_fit(r: &FitResult, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(path, (640, 420)).into_drawing_area();
    root.fill(&WHITE)?;

    let k_min = *r.k.first().unwrap();
    let k_max = *r.k.last().unwrap();
    let all_iv: Vec<f64> = r.iv_market.iter().chain(r.iv_fit.iter()).copied().collect();
    let iv_min = all_iv.iter().cloned().fold(f64::INFINITY, f64::min) * 0.92;
    let iv_max = all_iv.iter().cloned().fold(0.0_f64, f64::max) * 1.08;

    let title = format!(
        "T={:.4} (surface fit) avg price err: {:.1} bps, cal viol: {}",
        r.t_expiry, r.avg_price_err_bps, r.calendar_violations
    );

    let mut chart = ChartBuilder::on(&root)
        .caption(&title, ("sans-serif", 13))
        .margin(10)
        .x_label_area_size(35)
        .y_label_area_size(55)
        .build_cartesian_2d(k_min..k_max, iv_min..iv_max)?;

    chart
        .configure_mesh()
        .x_desc("log-moneyness")
        .y_desc("implied volatility")
        .draw()?;

    chart.draw_series(
        r.k.iter()
            .zip(r.iv_market.iter())
            .map(|(&k, &iv)| Circle::new((k, iv), 3, BLACK.filled())),
    )?
    .label("Market")
    .legend(|(x, y)| Circle::new((x + 10, y), 3, BLACK.filled()));

    let n_dense = 200;
    let dk = (k_max - k_min) / (n_dense - 1) as f64;
    let dense_k: Vec<f64> = (0..n_dense).map(|i| k_min + i as f64 * dk).collect();
    let dense_w = ssvi::total_variance_slice(&dense_k, r.theta, r.eta, r.gamma, r.rho);
    let dense_iv: Vec<f64> = dense_w
        .iter()
        .map(|&w| (w / r.t_expiry).max(0.0).sqrt())
        .collect();

    chart.draw_series(LineSeries::new(
        dense_k.iter().zip(dense_iv.iter()).map(|(&k, &iv)| (k, iv)),
        RED.stroke_width(2),
    ))?
    .label("SSVI Fit")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2)));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
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
        if let Err(e) = plot_fit(r, &path) {
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
