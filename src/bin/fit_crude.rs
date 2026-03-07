/// Fit SSVI surface to CBOE option data using the crude solver (4D Nelder-Mead).
///
/// Usage: cargo run --bin fit_crude -- <ticker> <date>
///   e.g. cargo run --bin fit_crude -- spx 2026-03-07
///
/// Three sections:
///   parse_cboe  -- CSV -> Vec<CrudeSlice> sorted by expiry
///   fit_surface -- crude_solver::calibrate_surface with calendar penalty
///   report      -- stdout summary table + SVG plots to documents/

use essvi::crude_solver::{CrudeCalibConfig, SliceInput, calibrate_surface};
use essvi::fit_common::{FitResult, plot_fit};
use essvi::model::ssvi;
use std::collections::BTreeMap;
use std::fs;

// ── Local types ──────────────────────────────────────────────

/// Parsed slice from CBOE CSV, carrying both total variance (for fitting)
/// and implied volatility (for plotting).
struct CrudeSlice {
    t: f64,
    k: Vec<f64>,
    w: Vec<f64>,
    iv: Vec<f64>,
}

/// Output of crude solver calibration for one slice.
struct FitOutput {
    t: f64,
    theta: f64,
    eta: f64,
    gamma: f64,
    rho: f64,
    sse: f64,
    converged: bool,
    k: Vec<f64>,
    iv_market: Vec<f64>,
}

// ── 1. Parse ─────────────────────────────────────────────────

/// Parse a CBOE CSV into sorted Vec<CrudeSlice>.
///
/// Extracts log_moneyness, total_variance (for fitting), and IV (for plots).
/// Groups by DTE, skips DTE < 1 and slices with < 5 points.
/// Focuses fitting range to log_moneyness in [-1.0, 1.0].
fn parse_cboe(path: &str) -> Vec<CrudeSlice> {
    let content = fs::read_to_string(path).expect("read csv");
    let mut lines = content.lines();
    let header: Vec<&str> = lines.next().expect("header").split(',').collect();

    let col = |name: &str| -> usize {
        header
            .iter()
            .position(|&h| h.trim() == name)
            .unwrap_or_else(|| panic!("missing column: {}", name))
    };
    let i_dte = col("dte");
    let i_log_m = col("log_moneyness");
    let i_total_var = col("total_variance");
    let i_c_iv = col("c_iv");
    let i_p_iv = col("p_iv");

    // Collect (log_m, total_variance, iv) grouped by DTE
    let mut groups: BTreeMap<u32, Vec<(f64, f64, f64)>> = BTreeMap::new();

    for line in lines {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() <= i_total_var.max(i_log_m).max(i_c_iv).max(i_p_iv) {
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
        let total_var: f64 = match f[i_total_var].trim().parse() {
            Ok(v) if v > 0.0 => v,
            _ => continue,
        };

        // IV using 3-zone rule (for plot labels)
        let min_iv = 0.01;
        let iv: f64 = if log_m < -0.1 {
            match f[i_p_iv].trim().parse::<f64>() {
                Ok(v) if v > min_iv => v,
                _ => continue,
            }
        } else if log_m > 0.1 {
            match f[i_c_iv].trim().parse::<f64>() {
                Ok(v) if v > min_iv => v,
                _ => continue,
            }
        } else {
            let p: f64 = match f[i_p_iv].trim().parse() {
                Ok(v) if v > min_iv => v,
                _ => continue,
            };
            let c: f64 = match f[i_c_iv].trim().parse() {
                Ok(v) if v > min_iv => v,
                _ => continue,
            };
            (p + c) / 2.0
        };

        groups.entry(dte as u32).or_default().push((log_m, total_var, iv));
    }

    // Convert to CrudeSlice sorted by expiry
    let mut slices: Vec<CrudeSlice> = Vec::new();
    for (&dte, pts) in &groups {
        if pts.len() < 5 {
            continue;
        }
        let t = dte as f64 / 365.0;
        let mut sorted = pts.clone();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        // Focus fitting range
        sorted.retain(|&(k, _, _)| k >= -1.0 && k <= 1.0);
        if sorted.len() < 5 {
            continue;
        }
        let k: Vec<f64> = sorted.iter().map(|(k, _, _)| *k).collect();
        let w: Vec<f64> = sorted.iter().map(|(_, w, _)| *w).collect();
        let iv: Vec<f64> = sorted.iter().map(|(_, _, iv)| *iv).collect();
        slices.push(CrudeSlice { t, k, w, iv });
    }
    slices.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
    slices
}

// ── 2. Fit ───────────────────────────────────────────────────

/// Calibrate all slices via crude_solver::calibrate_surface.
fn fit_surface(slices: &[CrudeSlice]) -> Vec<FitOutput> {
    let config = CrudeCalibConfig::default();

    // Build SliceInput references
    let slice_inputs: Vec<SliceInput> = slices
        .iter()
        .map(|s| SliceInput {
            t: s.t,
            k: &s.k,
            w: &s.w,
        })
        .collect();

    println!("=== Crude Solver: calibrating {} slices ===\n", slices.len());

    let results = calibrate_surface(&slice_inputs, &config);

    // Zip results with input slices
    let mut outputs = Vec::with_capacity(results.len());
    for (res, slice) in results.iter().zip(slices.iter()) {
        let dte = (slice.t * 365.0).round() as u32;
        println!(
            "  DTE={:>4}d (T={:.4}): theta={:.6}, eta={:.3}, gamma={:.3}, rho={:+.3}, SSE={:.2e}, converged={}",
            dte, slice.t, res.theta, res.eta, res.gamma, res.rho, res.sse,
            if res.converged { "yes" } else { "NO" }
        );
        outputs.push(FitOutput {
            t: slice.t,
            theta: res.theta,
            eta: res.eta,
            gamma: res.gamma,
            rho: res.rho,
            sse: res.sse,
            converged: res.converged,
            k: slice.k.clone(),
            iv_market: slice.iv.clone(),
        });
    }

    outputs
}

// ── 3. Report ────────────────────────────────────────────────

/// Generate SVG plots and print summary table.
fn report(results: &[FitOutput], ticker: &str, date: &str) {
    let name = format!("{}_{}_crude", ticker, date);
    let plot_dir = format!("documents/data-plots/{}", name);
    fs::create_dir_all(&plot_dir).expect("create plot dir");

    // Generate per-slice SVG plots
    for r in results {
        let dte = (r.t * 365.0).round() as u32;

        // Compute fitted IV from fitted SSVI parameters
        let w_fit = ssvi::total_variance_slice(&r.k, r.theta, r.eta, r.gamma, r.rho);
        let iv_fit: Vec<f64> = w_fit.iter().map(|&w| (w / r.t).max(0.0).sqrt()).collect();

        // Compute error metrics
        let iv_errors: Vec<f64> = iv_fit
            .iter()
            .zip(r.iv_market.iter())
            .map(|(f, m)| (f - m).abs())
            .collect();
        let max_iv_err = iv_errors.iter().cloned().fold(0.0_f64, f64::max);
        let rmse_iv =
            (iv_errors.iter().map(|e| e * e).sum::<f64>() / iv_errors.len() as f64).sqrt();
        let avg_err = iv_errors.iter().sum::<f64>() / iv_errors.len() as f64;

        let fit_result = FitResult {
            t_expiry: r.t,
            eta: r.eta,
            gamma: r.gamma,
            rho: r.rho,
            theta: r.theta,
            phi: ssvi::phi(r.theta, r.eta, r.gamma),
            no_arb_usage: r.eta * (1.0 + r.rho.abs()),
            converged: r.converged,
            max_iv_err_bps: max_iv_err * 10000.0,
            rmse_iv_bps: rmse_iv * 10000.0,
            avg_price_err_bps: avg_err * 10000.0,
            calendar_violations: 0,
            max_calendar_violation_bps: 0.0,
            k: r.k.clone(),
            iv_market: r.iv_market.clone(),
            iv_fit,
        };

        let path = format!("{}/fit_dte_{}.svg", plot_dir, dte);
        let title = format!(
            "{} DTE={} (T={:.4}) crude solver",
            ticker.to_uppercase(),
            dte,
            r.t
        );
        if let Err(e) = plot_fit(&fit_result, &path, &title) {
            eprintln!("Plot error DTE={}: {}", dte, e);
        }
    }

    // Print summary table
    println!(
        "\n=== Crude Solver Results: {} {} ===\n",
        ticker.to_uppercase(),
        date
    );
    println!(
        "| {:>4} | {:>8} | {:>8} | {:>6} | {:>6} | {:>7} | {:>10} | {:>9} |",
        "DTE", "T", "theta", "eta", "gamma", "rho", "SSE", "converged"
    );
    println!(
        "|{:-^6}|{:-^10}|{:-^10}|{:-^8}|{:-^8}|{:-^9}|{:-^12}|{:-^11}|",
        "", "", "", "", "", "", "", ""
    );
    for r in results {
        let dte = (r.t * 365.0).round() as u32;
        println!(
            "| {:>4} | {:>8.4} | {:>8.6} | {:>6.3} | {:>6.3} | {:>+7.3} | {:>10.2e} | {:>9} |",
            dte,
            r.t,
            r.theta,
            r.eta,
            r.gamma,
            r.rho,
            r.sse,
            if r.converged { "yes" } else { "NO" }
        );
    }

    println!("\nPlots: {}/", plot_dir);
}

// ── 4. Main ──────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cargo run --bin fit_crude -- <ticker> <date>");
        eprintln!("  e.g. cargo run --bin fit_crude -- spx 2026-03-07");
        std::process::exit(1);
    }
    let ticker = &args[1];
    let date = &args[2];
    let csv_path = format!("data/cboe/{}/{}.csv", ticker, date);

    println!("Parsing {}...", csv_path);
    let slices = parse_cboe(&csv_path);
    println!("Found {} expiry slices\n", slices.len());

    let results = fit_surface(&slices);
    println!("\nFitted {} slices successfully", results.len());

    report(&results, ticker, date);
}
