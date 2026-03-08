/// Fit SSVI surface to Kaggle SPY option data.
///
/// Usage: cargo run --bin fit_kaggle -- 2020-03-13
///
/// Three functions:
///   parse_kaggle  – CSV → Vec<SliceData> sorted by expiry
///   fit_surface   – SSVI surface fit with calendar penalty
///   report        – markdown + SVG plots to documents/
use essvi::calibration::{
    CalibrationConfig, CalibrationInput, CalibrationResult, PrevSlice, calibrate,
    calibrate_with_calendar_penalty,
};
use essvi::fit_common::{FitResult, SliceData, plot_fit};
use essvi::ssvi;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

// ── 1. Parse ────────────────────────────────────────────────

/// Parse a Kaggle SPY CSV into sorted Vec<SliceData>.
///
/// Groups by DTE, applies the 3-zone IV rule:
///   log_m < -0.1  → P_IV
///   -0.1..=0.1    → mean(P_IV, C_IV)
///   log_m > 0.1   → C_IV
/// Skips DTE=0 rows and rows with missing IV.
fn parse_kaggle(path: &str) -> Vec<SliceData> {
    let content = fs::read_to_string(path).expect("read csv");
    let mut lines = content.lines();
    let header: Vec<&str> = lines.next().expect("header").split(',').collect();

    let col = |name: &str| -> usize {
        header
            .iter()
            .position(|&h| h.trim() == name)
            .unwrap_or_else(|| panic!("missing column: {}", name))
    };
    let i_dte = col("DTE");
    let i_c_iv = col("C_IV");
    let i_p_iv = col("P_IV");
    let i_log_m = col("LOG_MONEYNESS");

    // Collect (dte, log_m, iv) grouped by DTE
    let mut groups: BTreeMap<u32, Vec<(f64, f64)>> = BTreeMap::new();

    for line in lines {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() <= i_log_m {
            continue;
        }
        let dte: f64 = match f[i_dte].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if dte < 1.0 {
            continue;
        }
        let log_m: f64 = match f[i_log_m].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let iv: f64 = if log_m < -0.1 {
            match f[i_p_iv].trim().parse::<f64>() {
                Ok(v) if v > 0.0 => v,
                _ => continue,
            }
        } else if log_m > 0.1 {
            match f[i_c_iv].trim().parse::<f64>() {
                Ok(v) if v > 0.0 => v,
                _ => continue,
            }
        } else {
            let p: f64 = match f[i_p_iv].trim().parse() {
                Ok(v) if v > 0.0 => v,
                _ => continue,
            };
            let c: f64 = match f[i_c_iv].trim().parse() {
                Ok(v) if v > 0.0 => v,
                _ => continue,
            };
            (p + c) / 2.0
        };

        groups.entry(dte as u32).or_default().push((log_m, iv));
    }

    // Convert to SliceData sorted by expiry
    let mut slices: Vec<SliceData> = Vec::new();
    for (&dte, pts) in &groups {
        if pts.len() < 5 {
            continue;
        }
        let t = dte as f64 / 365.0;
        let mut sorted: Vec<(f64, f64)> = pts.clone();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        // Focus fitting range to -1.0..1.0
        sorted.retain(|&(k, _)| k >= -1.0 && k <= 1.0);
        if sorted.len() < 5 {
            continue;
        }
        let k: Vec<f64> = sorted.iter().map(|(k, _)| *k).collect();
        let iv: Vec<f64> = sorted.iter().map(|(_, v)| *v).collect();
        slices.push(SliceData { t_expiry: t, k, iv });
    }
    slices.sort_by(|a, b| a.t_expiry.partial_cmp(&b.t_expiry).unwrap());
    slices
}

// ── 2. Fit ──────────────────────────────────────────────────

/// Fit SSVI surface with calendar arbitrage penalty.
///
/// k_penalty: -1.5 to 0.5 step 0.05, lambda = 100.
/// Step 1: unconstrained per-slice.
/// Step 2: sequential refit with calendar penalty from previous slice.
fn fit_surface(slices: &[SliceData]) -> Vec<FitResult> {
    let config = CalibrationConfig::default();
    let k_penalty: Vec<f64> = {
        let mut v = Vec::new();
        let mut k = -0.8_f64;
        while k <= 0.2 + 1e-9 {
            v.push(k);
            k += 0.02;
        }
        v
    };
    let lambda = 50.0;

    // Step 1: unconstrained fit for initial guesses
    println!("=== Step 1: Unconstrained per-slice fit ===");
    let mut unconstrained: Vec<CalibrationResult> = Vec::new();

    for slice in slices {
        let t = slice.t_expiry;

        let w_market: Vec<f64> = slice.iv.iter().map(|&s| s * s * t).collect();

        let atm_idx = slice.k
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (*a - 0.0_f64).abs().partial_cmp(&(*b - 0.0_f64).abs()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(slice.k.len() / 2);
        let k_star = slice.k[atm_idx];
        let atm_vol = slice.iv[atm_idx];
        let theta_star = atm_vol * atm_vol * t;

        let weights: Vec<f64> = slice.k
            .iter()
            .map(|&k| if k >= -0.15 && k <= 0.15 { 3.0 } else { 1.0 })
            .collect();

        let input = CalibrationInput {
            k_slice: &slice.k,
            w_market: &w_market,
            theta_star,
            k_star,
            weights: Some(&weights),
        };

        match calibrate(&input, &config) {
            Ok(res) => {
                let w_at_kstar = ssvi::total_variance(k_star, res.theta, res.eta, res.gamma, res.rho);
                println!(
                    "  DTE={:.0}d (T={:.4}): k*={:.4}, θ*={:.6}, w(k*)={:.6}, gap={:.2e}, eta={:.4}, rho={:.4}",
                    t * 365.0, t, k_star, theta_star, w_at_kstar,
                    (w_at_kstar - theta_star).abs(), res.eta, res.rho
                );
                unconstrained.push(res);
            }
            Err(e) => {
                eprintln!("  FAILED DTE={:.0}d: {}", t * 365.0, e);
                // Push a fallback so indices stay aligned
                unconstrained.push(CalibrationResult {
                    eta: 0.5,
                    gamma: 0.5,
                    rho: -0.3,
                    theta: theta_star,
                    optimizer: essvi::solver::nelder_mead::NelderMeadResult {
                        x: vec![0.5, 0.5, -0.3],
                        f: f64::INFINITY,
                        iterations: 0,
                        converged: false,
                    },
                });
            }
        }
    }

    // Step 2: surface fit with calendar penalty
    println!(
        "\n=== Step 2: Surface fit (lambda={}, k_penalty: -0.8..0.2 step 0.02) ===",
        lambda
    );
    let mut results: Vec<FitResult> = Vec::new();

    for (i, slice) in slices.iter().enumerate() {
        let t = slice.t_expiry;

        let w_market: Vec<f64> = slice.iv.iter().map(|&s| s * s * t).collect();

        let atm_idx = slice.k
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (*a - 0.0_f64).abs().partial_cmp(&(*b - 0.0_f64).abs()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(slice.k.len() / 2);
        let k_star = slice.k[atm_idx];
        let atm_vol = slice.iv[atm_idx];
        let theta_star = atm_vol * atm_vol * t;

        let weights: Vec<f64> = slice.k
            .iter()
            .map(|&k| if k >= -0.15 && k <= 0.15 { 3.0 } else { 1.0 })
            .collect();

        let input = CalibrationInput {
            k_slice: &slice.k,
            w_market: &w_market,
            theta_star,
            k_star,
            weights: Some(&weights),
        };

        let res = if i == 0 {
            calibrate(&input, &config)
        } else {
            let prev_fr = &results[i - 1];
            let prev = PrevSlice {
                theta: prev_fr.theta,
                eta: prev_fr.eta,
                gamma: prev_fr.gamma,
                rho: prev_fr.rho,
            };
            let unc = &unconstrained[i];
            let init = [unc.eta, unc.gamma, unc.rho];
            calibrate_with_calendar_penalty(&input, &config, &prev, &k_penalty, lambda, &init)
        };

        match res {
            Ok(res) => {
                let w_fit =
                    ssvi::total_variance_slice(&slice.k, res.theta, res.eta, res.gamma, res.rho);
                let iv_fit: Vec<f64> =
                    w_fit.iter().map(|&w| (w / t).max(0.0).sqrt()).collect();

                let iv_errors: Vec<f64> = iv_fit
                    .iter()
                    .zip(slice.iv.iter())
                    .map(|(f, m)| (f - m).abs())
                    .collect();
                let max_iv_err =
                    iv_errors.iter().cloned().fold(0.0_f64, f64::max);
                let rmse_iv =
                    (iv_errors.iter().map(|e| e * e).sum::<f64>() / iv_errors.len() as f64).sqrt();
                let avg_price_err =
                    iv_errors.iter().sum::<f64>() / iv_errors.len() as f64;

                // Calendar violations
                let (cal_viol, max_cal_bps) = if i > 0 {
                    let prev_fr = &results[i - 1];
                    let mut violations = 0usize;
                    let mut max_v = 0.0_f64;
                    for &k in &k_penalty {
                        let w_prev = ssvi::total_variance(
                            k, prev_fr.theta, prev_fr.eta, prev_fr.gamma, prev_fr.rho,
                        );
                        let w_cur = ssvi::total_variance(
                            k, res.theta, res.eta, res.gamma, res.rho,
                        );
                        if w_prev > w_cur + 1e-14 {
                            violations += 1;
                            let iv_p = (w_prev / t).max(0.0).sqrt();
                            let iv_c = (w_cur / t).max(0.0).sqrt();
                            max_v = max_v.max((iv_p - iv_c).abs() * 10000.0);
                        }
                    }
                    (violations, max_v)
                } else {
                    (0, 0.0)
                };

                println!(
                    "  DTE={:.0}d: max_err={:.1} bps, RMSE={:.1} bps, cal_viol={}",
                    t * 365.0,
                    max_iv_err * 10000.0,
                    rmse_iv * 10000.0,
                    cal_viol
                );

                results.push(FitResult {
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
                    calendar_violations: cal_viol,
                    max_calendar_violation_bps: max_cal_bps,
                    k: slice.k.clone(),
                    iv_market: slice.iv.clone(),
                    iv_fit,
                });
            }
            Err(e) => {
                eprintln!("  FAILED DTE={:.0}d: {}", t * 365.0, e);
            }
        }
    }

    results
}

// ── 3. Report ───────────────────────────────────────────────

/// Generate markdown report with SVG plots.
fn report(results: &[FitResult], name: &str) {
    let plot_dir = format!("documents/data-plots/{}", name);
    fs::create_dir_all(&plot_dir).expect("create plot dir");

    // Generate per-slice SVG plots
    for r in results {
        let dte = (r.t_expiry * 365.0).round() as u32;
        let path = format!("{}/fit_dte_{}.svg", plot_dir, dte);
        let title = format!(
            "DTE={} (T={:.4}) avg err: {:.1} bps",
            dte, r.t_expiry, r.avg_price_err_bps
        );
        if let Err(e) = plot_fit(r, &path, &title) {
            eprintln!("Plot error DTE={}: {}", dte, e);
        }
    }

    // Write markdown
    let mut md = String::new();
    md.push_str(&format!("# SSVI Kaggle Fit: {}\n\n", name));
    md.push_str("Surface calibration with calendar arbitrage penalty.\n\n");
    md.push_str("- k penalty range: -0.8 to 0.2, step 0.02\n");
    md.push_str("- lambda: 50\n");
    md.push_str("- IV rule: P_IV (k < -0.1), mean(P_IV,C_IV) (-0.1..0.1), C_IV (k > 0.1)\n\n");

    // Summary table
    md.push_str("## Calibration Summary\n\n");
    md.push_str("| DTE | T | max err (bps) | RMSE (bps) | avg err (bps) | eta | gamma | rho | phi | cal viol | converged |\n");
    md.push_str("|----:|------:|--------------:|-----------:|--------------:|------:|------:|------:|------:|---------:|:---------:|\n");
    for r in results {
        let dte = (r.t_expiry * 365.0).round() as u32;
        md.push_str(&format!(
            "| {} | {:.4} | {:.1} | {:.1} | {:.1} | {:.4} | {:.4} | {:.4} | {:.3} | {} | {} |\n",
            dte,
            r.t_expiry,
            r.max_iv_err_bps,
            r.rmse_iv_bps,
            r.avg_price_err_bps,
            r.eta,
            r.gamma,
            r.rho,
            r.phi,
            r.calendar_violations,
            if r.converged { "yes" } else { "**no**" }
        ));
    }
    md.push_str("\n");

    // Fit plots
    md.push_str("## Fit Plots\n\n");
    for r in results {
        let dte = (r.t_expiry * 365.0).round() as u32;
        md.push_str(&format!("### DTE = {} (T = {:.4})\n\n", dte, r.t_expiry));
        md.push_str(&format!(
            "max err: {:.1} bps | RMSE: {:.1} bps | eta={:.4}, gamma={:.4}, rho={:.4} | cal viol: {}\n\n",
            r.max_iv_err_bps, r.rmse_iv_bps, r.eta, r.gamma, r.rho, r.calendar_violations
        ));
        md.push_str(&format!(
            "![DTE={}]({}/fit_dte_{}.svg)\n\n",
            dte, name, dte
        ));
    }

    let report_path = format!("documents/data-plots/{}-report.md", name);
    let mut file = fs::File::create(&report_path).expect("create report file");
    file.write_all(md.as_bytes()).expect("write report");

    println!("\nReport: {}", report_path);
    println!("Plots:  {}/", plot_dir);
}

// ── 4. Main ─────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin fit_kaggle -- <date>");
        eprintln!("  e.g. cargo run --bin fit_kaggle -- 2020-03-13");
        std::process::exit(1);
    }
    let name = &args[1];
    let csv_path = format!("data/kaggle/spy/{}.csv", name);

    println!("Parsing {}...", csv_path);
    let slices = parse_kaggle(&csv_path);
    println!("Found {} expiry slices\n", slices.len());

    let results = fit_surface(&slices);
    println!("\nFitted {} slices successfully", results.len());

    report(&results, name);
}
