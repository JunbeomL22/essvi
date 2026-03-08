/// Fit SSVI surface to Kaggle SPY option data using the direct solver (4D Nelder-Mead).
///
/// Usage: cargo run --bin direct_fit_kaggle -- <date>
///   e.g. cargo run --bin direct_fit_kaggle -- 2020-03-13
///
/// Three functions:
///   parse_kaggle  – CSV → Vec<DirectSlice> sorted by expiry
///   fit_surface   – direct_solver::calibrate_surface with calendar penalty
///   report        – markdown + SVG plots to documents/
use essvi::direct_solver::{DirectCalibConfig, SliceInput, calibrate_surface, validate_butterfly};
use essvi::fit_common::{FitResult, plot_fit};
use essvi::model::ssvi;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

// ── Local type ──────────────────────────────────────────────

/// Parsed slice from Kaggle CSV, carrying both total variance (for fitting)
/// and implied volatility (for plotting).
struct DirectSlice {
    t: f64,
    k: Vec<f64>,
    w: Vec<f64>,
    iv: Vec<f64>,
}

// ── 1. Parse ────────────────────────────────────────────────

/// Parse a Kaggle SPY CSV into sorted Vec<DirectSlice>.
///
/// Groups by DTE, applies the 3-zone IV rule:
///   log_m < -0.1  → P_IV
///   -0.1..=0.1    → mean(P_IV, C_IV)
///   log_m > 0.1   → C_IV
/// Computes total variance as w = iv^2 * t.
/// Skips DTE<1 rows and rows with missing IV.
fn parse_kaggle(path: &str) -> Vec<DirectSlice> {
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

    // Convert to DirectSlice sorted by expiry
    let mut slices: Vec<DirectSlice> = Vec::new();
    for (&dte, pts) in &groups {
        if pts.len() < 5 {
            continue;
        }
        let t = dte as f64 / 365.0;
        let mut sorted: Vec<(f64, f64)> = pts.clone();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        sorted.retain(|&(k, _)| k >= -1.0 && k <= 1.0);
        if sorted.len() < 5 {
            continue;
        }
        let k: Vec<f64> = sorted.iter().map(|(k, _)| *k).collect();
        let iv: Vec<f64> = sorted.iter().map(|(_, v)| *v).collect();
        let w: Vec<f64> = iv.iter().map(|&s| s * s * t).collect();
        slices.push(DirectSlice { t, k, w, iv });
    }
    slices.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
    slices
}

// ── 2. Fit ──────────────────────────────────────────────────

/// Calibrate all slices via direct_solver::calibrate_surface.
fn fit_surface(slices: &[DirectSlice]) -> Vec<FitResult> {
    let config = DirectCalibConfig::default();

    let slice_inputs: Vec<SliceInput> = slices
        .iter()
        .map(|s| SliceInput {
            t: s.t,
            k: &s.k,
            w: &s.w,
        })
        .collect();

    println!("=== Direct Solver: calibrating {} slices ===\n", slices.len());

    let calib_results = calibrate_surface(&slice_inputs, &config);

    // Build FitResult for each slice
    let k_butterfly: Vec<f64> = {
        let mut v = Vec::new();
        let mut k = -1.5_f64;
        while k <= 0.5 + 1e-9 {
            v.push(k);
            k += 0.025;
        }
        v
    };

    let mut results: Vec<FitResult> = Vec::new();

    for (i, (res, slice)) in calib_results.iter().zip(slices.iter()).enumerate() {
        let t = slice.t;
        let dte = (t * 365.0).round() as u32;

        let w_fit = ssvi::total_variance_slice(&slice.k, res.theta, res.eta, res.gamma, res.rho);
        let iv_fit: Vec<f64> = w_fit.iter().map(|&w| (w / t).max(0.0).sqrt()).collect();

        let iv_errors: Vec<f64> = iv_fit
            .iter()
            .zip(slice.iv.iter())
            .map(|(f, m)| (f - m).abs())
            .collect();
        let max_iv_err = iv_errors.iter().cloned().fold(0.0_f64, f64::max);
        let rmse_iv =
            (iv_errors.iter().map(|e| e * e).sum::<f64>() / iv_errors.len() as f64).sqrt();
        let avg_err = iv_errors.iter().sum::<f64>() / iv_errors.len() as f64;

        // Calendar violations against previous slice
        let (cal_viol, max_cal_bps) = if i > 0 {
            let prev = &results[i - 1];
            let mut violations = 0usize;
            let mut max_v = 0.0_f64;
            for &k in &k_butterfly {
                let w_prev =
                    ssvi::total_variance(k, prev.theta, prev.eta, prev.gamma, prev.rho);
                let w_cur =
                    ssvi::total_variance(k, res.theta, res.eta, res.gamma, res.rho);
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

        // Butterfly validation
        let bf = validate_butterfly(res.theta, res.eta, res.gamma, res.rho, &k_butterfly);
        let bf_violations = bf.iter().filter(|b| !b.valid).count();

        println!(
            "  DTE={:>4}d: max_err={:.1} bps, RMSE={:.1} bps, cal_viol={}, butterfly_viol={}, converged={}",
            dte,
            max_iv_err * 10000.0,
            rmse_iv * 10000.0,
            cal_viol,
            bf_violations,
            if res.converged { "yes" } else { "NO" }
        );

        results.push(FitResult {
            t_expiry: t,
            eta: res.eta,
            gamma: res.gamma,
            rho: res.rho,
            theta: res.theta,
            phi: ssvi::phi(res.theta, res.eta, res.gamma),
            no_arb_usage: res.eta * (1.0 + res.rho.abs()),
            converged: res.converged,
            max_iv_err_bps: max_iv_err * 10000.0,
            rmse_iv_bps: rmse_iv * 10000.0,
            avg_price_err_bps: avg_err * 10000.0,
            calendar_violations: cal_viol,
            max_calendar_violation_bps: max_cal_bps,
            k: slice.k.clone(),
            iv_market: slice.iv.clone(),
            iv_fit,
        });
    }

    results
}

// ── 3. Report ───────────────────────────────────────────────

/// Generate markdown report with SVG plots.
fn report(results: &[FitResult], name: &str) {
    let plot_dir = format!("documents/data-plots/{}_direct", name);
    fs::create_dir_all(&plot_dir).expect("create plot dir");

    let dir_name = format!("{}_direct", name);

    // Generate per-slice SVG plots
    for r in results {
        let dte = (r.t_expiry * 365.0).round() as u32;
        let path = format!("{}/fit_dte_{}.svg", plot_dir, dte);
        let title = format!(
            "SPY DTE={} (T={:.4}) direct solver, avg err: {:.1} bps",
            dte, r.t_expiry, r.avg_price_err_bps
        );
        if let Err(e) = plot_fit(r, &path, &title) {
            eprintln!("Plot error DTE={}: {}", dte, e);
        }
    }

    // Write markdown
    let mut md = String::new();
    md.push_str(&format!(
        "# SSVI Direct Solver Fit: Kaggle SPY ({})\n\n",
        name
    ));
    md.push_str("4D Nelder-Mead direct solver with algebraic no-arb barrier.\n\n");
    md.push_str("- Objective: pure SSE on total variance\n");
    md.push_str("- No-arb barrier: eta*(1+|rho|) <= 2\n");
    md.push_str("- Calendar penalty: theta monotonicity via lambda_calendar\n");
    md.push_str("- Butterfly: post-hoc validation\n");
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
            dte, dir_name, dte
        ));
    }

    let report_path = format!("documents/data-plots/{}-report.md", dir_name);
    let mut file = fs::File::create(&report_path).expect("create report file");
    file.write_all(md.as_bytes()).expect("write report");

    println!("\nReport: {}", report_path);
    println!("Plots:  {}/", plot_dir);
}

// ── 4. Main ─────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin direct_fit_kaggle -- <date>");
        eprintln!("  e.g. cargo run --bin direct_fit_kaggle -- 2020-03-13");
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
