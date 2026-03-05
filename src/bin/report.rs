/// Generate SSVI fit quality report across parameter grid:
///   T (expiry), skew slope, moneyness width, k*

use essvi::calibration::{calibrate, solve_theta, CalibrationInput};
use essvi::nelder_mead::NelderMeadConfig;
use essvi::ssvi;
use plotters::prelude::*;
use std::fs;
use std::io::Write;

// ── Scenario parameters ─────────────────────────────────────

struct Scenario {
    label: String,
    t_expiry: f64,
    slope: f64,       // put-side vol increase per unit |k|
    width: f64,       // moneyness range: [-width, +width*0.4]
    k_star: f64,
    atm_vol: f64,
}

fn make_market_data(s: &Scenario) -> (Vec<f64>, Vec<f64>) {
    // 20 points from -width to +width*0.4
    let k_lo = -s.width;
    let k_hi = s.width * 0.4;
    let k_slice: Vec<f64> = (0..20)
        .map(|i| k_lo + (i as f64) * (k_hi - k_lo) / 19.0)
        .collect();

    // Smile shape: quadratic + linear skew
    //   σ(k) = atm_vol + slope * (-k).max(0) + 0.1 * slope * k.max(0) + curvature * k²
    let curvature = s.slope * 0.15; // mild convexity
    let iv: Vec<f64> = k_slice
        .iter()
        .map(|&k| {
            let skew = if k < 0.0 {
                s.slope * (-k)
            } else {
                0.1 * s.slope * k
            };
            let smile = curvature * k * k;
            (s.atm_vol + skew + smile).max(0.01)
        })
        .collect();

    let w_market: Vec<f64> = iv.iter().map(|&sigma| sigma * sigma * s.t_expiry).collect();
    (k_slice, w_market)
}

struct FitResult {
    scenario: Scenario,
    eta: f64,
    gamma: f64,
    rho: f64,
    theta: f64,
    phi: f64,
    converged: bool,
    iterations: usize,
    sse: f64,
    max_iv_err: f64,
    rmse_iv: f64,
    k_slice: Vec<f64>,
    iv_market: Vec<f64>,
    iv_fit: Vec<f64>,
}

fn run_scenario(s: Scenario) -> Option<FitResult> {
    let (k_slice, w_market) = make_market_data(&s);
    let theta_star = s.atm_vol * s.atm_vol * s.t_expiry;

    let input = CalibrationInput {
        k_slice: &k_slice,
        w_market: &w_market,
        theta_star,
        k_star: s.k_star,
    };

    let config = NelderMeadConfig::default();
    let res = calibrate(&input, &config)?;

    let w_fit = ssvi::total_variance_slice(&k_slice, res.theta, res.eta, res.gamma, res.rho);

    let iv_market: Vec<f64> = w_market.iter().map(|&w| (w / s.t_expiry).sqrt()).collect();
    let iv_fit: Vec<f64> = w_fit.iter().map(|&w| (w / s.t_expiry).sqrt()).collect();

    let iv_errors: Vec<f64> = iv_fit
        .iter()
        .zip(iv_market.iter())
        .map(|(f, m)| (f - m).abs())
        .collect();
    let max_iv_err = iv_errors.iter().cloned().fold(0.0_f64, f64::max);
    let rmse_iv = (iv_errors.iter().map(|e| e * e).sum::<f64>() / iv_errors.len() as f64).sqrt();

    Some(FitResult {
        phi: ssvi::phi(res.theta, res.eta, res.gamma),
        eta: res.eta,
        gamma: res.gamma,
        rho: res.rho,
        theta: res.theta,
        converged: res.optimizer.converged,
        iterations: res.optimizer.iterations,
        sse: res.optimizer.f,
        max_iv_err,
        rmse_iv,
        k_slice,
        iv_market,
        iv_fit,
        scenario: s,
    })
}

// ── Plot generation ─────────────────────────────────────────

fn plot_fit(result: &FitResult, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(path, (600, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let k_min = result.k_slice.first().copied().unwrap_or(-1.0);
    let k_max = result.k_slice.last().copied().unwrap_or(1.0);
    let iv_all: Vec<f64> = result
        .iv_market
        .iter()
        .chain(result.iv_fit.iter())
        .copied()
        .collect();
    let iv_min = iv_all.iter().cloned().fold(f64::INFINITY, f64::min) * 0.95;
    let iv_max = iv_all.iter().cloned().fold(0.0_f64, f64::max) * 1.05;

    let title = format!(
        "T={}, slope={}, width={}, k*={}",
        result.scenario.t_expiry,
        result.scenario.slope,
        result.scenario.width,
        result.scenario.k_star
    );

    let mut chart = ChartBuilder::on(&root)
        .caption(&title, ("sans-serif", 16))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(k_min..k_max, iv_min..iv_max)?;

    chart
        .configure_mesh()
        .x_desc("Log-moneyness k")
        .y_desc("Implied Vol")
        .draw()?;

    // Market points
    chart.draw_series(
        result
            .k_slice
            .iter()
            .zip(result.iv_market.iter())
            .map(|(&k, &iv)| Circle::new((k, iv), 3, BLUE.filled())),
    )?
    .label("Market")
    .legend(|(x, y)| Circle::new((x + 10, y), 3, BLUE.filled()));

    // Fit line
    chart.draw_series(LineSeries::new(
        result
            .k_slice
            .iter()
            .zip(result.iv_fit.iter())
            .map(|(&k, &iv)| (k, iv)),
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

fn plot_heatmap(
    results: &[FitResult],
    t_vals: &[f64],
    slope_vals: &[f64],
    path: &str,
    title: &str,
    extractor: impl Fn(&FitResult) -> f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(path, (600, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let n_t = t_vals.len();
    let n_s = slope_vals.len();

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 16))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n_s, 0..n_t)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Slope index")
        .y_desc("T index")
        .x_labels(n_s)
        .y_labels(n_t)
        .x_label_formatter(&|&i| {
            if i < n_s {
                format!("{:.1}", slope_vals[i])
            } else {
                String::new()
            }
        })
        .y_label_formatter(&|&i| {
            if i < n_t {
                format!("{:.0e}", t_vals[i])
            } else {
                String::new()
            }
        })
        .draw()?;

    // Find value range
    let vals: Vec<f64> = results.iter().map(&extractor).collect();
    let v_max = vals.iter().cloned().fold(0.0_f64, f64::max);

    for (idx, r) in results.iter().enumerate() {
        let si = idx % n_s;
        let ti = idx / n_s;
        let val = extractor(r);
        let intensity = if v_max > 0.0 {
            (val / v_max).min(1.0)
        } else {
            0.0
        };
        // Green (good) to Red (bad)
        let color = RGBColor(
            (255.0 * intensity) as u8,
            (255.0 * (1.0 - intensity)) as u8,
            0,
        );
        chart.draw_series(std::iter::once(Rectangle::new(
            [(si, ti), (si + 1, ti + 1)],
            color.filled(),
        )))?;

        // Value label
        let label = format!("{:.0}", val * 10000.0); // in bps
        chart.draw_series(std::iter::once(Text::new(
            label,
            (si, ti),
            ("sans-serif", 11).into_font().color(&BLACK),
        )))?;
    }

    root.present()?;
    Ok(())
}

// ── Main ────────────────────────────────────────────────────

fn main() {
    let plot_dir = "documents/plots";
    fs::create_dir_all(plot_dir).expect("create plot dir");

    // Parameter grid
    let t_vals = [1.0, 0.1, 0.01, 0.001];
    let slope_vals = [0.2, 0.4, 0.6, 0.8, 1.0];
    let width_vals = [0.3, 0.5, 0.7];
    let kstar_vals = [0.0, 0.005, 0.01, -0.01];
    let atm_vol = 0.4;

    let mut all_results: Vec<FitResult> = Vec::new();
    let mut md = String::new();

    md.push_str("# SSVI Slice Fit Quality Report\n\n");
    md.push_str(&format!("ATM vol = {}\n\n", atm_vol));

    // ── Section 1: T × Slope grid (fixed width=0.5, k*=0.005) ─────

    md.push_str("## 1. Expiry (T) vs Skew Slope\n\n");
    md.push_str("Fixed: width=0.5, k\\*=0.005\n\n");
    md.push_str("| T | Slope | max IV err (bps) | RMSE IV (bps) | rho | phi | converged |\n");
    md.push_str("|---|-------|-----------------|--------------|-----|-----|----------|\n");

    let mut grid_results: Vec<FitResult> = Vec::new();

    for &t in &t_vals {
        for &slope in &slope_vals {
            let s = Scenario {
                label: format!("T={}_s={}", t, slope),
                t_expiry: t,
                slope,
                width: 0.5,
                k_star: 0.005,
                atm_vol,
            };
            if let Some(r) = run_scenario(s) {
                md.push_str(&format!(
                    "| {:.0e} | {:.1} | {:.1} | {:.1} | {:.3} | {:.2} | {} |\n",
                    r.scenario.t_expiry,
                    r.scenario.slope,
                    r.max_iv_err * 10000.0,
                    r.rmse_iv * 10000.0,
                    r.rho,
                    r.phi,
                    r.converged
                ));
                grid_results.push(r);
            }
        }
    }

    // Heatmap
    let _ = plot_heatmap(
        &grid_results,
        &t_vals,
        &slope_vals,
        &format!("{}/heatmap_t_slope.svg", plot_dir),
        "Max IV Error (bps) — T vs Slope",
        |r| r.max_iv_err,
    );
    md.push_str("\n![T vs Slope Heatmap](plots/heatmap_t_slope.svg)\n\n");

    // Individual fit plots for extreme corners
    let corners = [
        (1.0, 0.2, "mild_long"),
        (1.0, 1.0, "steep_long"),
        (0.001, 0.2, "mild_short"),
        (0.001, 1.0, "steep_short"),
    ];
    md.push_str("### Fit plots (corners of grid)\n\n");
    for (t, slope, name) in &corners {
        let s = Scenario {
            label: name.to_string(),
            t_expiry: *t,
            slope: *slope,
            width: 0.5,
            k_star: 0.005,
            atm_vol,
        };
        if let Some(r) = run_scenario(s) {
            let path = format!("{}/fit_{}.svg", plot_dir, name);
            let _ = plot_fit(&r, &path);
            md.push_str(&format!(
                "**T={}, slope={}** — max err: {:.0} bps\n\n",
                t,
                slope,
                r.max_iv_err * 10000.0
            ));
            md.push_str(&format!("![{}](plots/fit_{}.svg)\n\n", name, name));
        }
    }

    // ── Section 2: Moneyness width ─────────────────────────────

    md.push_str("## 2. Moneyness Width\n\n");
    md.push_str("Fixed: slope=0.6, k\\*=0.005\n\n");
    md.push_str("| T | Width | max IV err (bps) | RMSE IV (bps) | rho | phi |\n");
    md.push_str("|---|-------|-----------------|--------------|-----|-----|\n");

    for &t in &t_vals {
        for &width in &width_vals {
            let s = Scenario {
                label: format!("T={}_w={}", t, width),
                t_expiry: t,
                slope: 0.6,
                width,
                k_star: 0.005,
                atm_vol,
            };
            if let Some(r) = run_scenario(s) {
                md.push_str(&format!(
                    "| {:.0e} | {:.1} | {:.1} | {:.1} | {:.3} | {:.2} |\n",
                    r.scenario.t_expiry,
                    r.scenario.width,
                    r.max_iv_err * 10000.0,
                    r.rmse_iv * 10000.0,
                    r.rho,
                    r.phi,
                ));
            }
        }
    }

    // Width comparison plot at T=0.01
    md.push_str("\n### Width comparison (T=0.01, slope=0.6)\n\n");
    for &width in &width_vals {
        let s = Scenario {
            label: format!("w={}", width),
            t_expiry: 0.01,
            slope: 0.6,
            width,
            k_star: 0.005,
            atm_vol,
        };
        if let Some(r) = run_scenario(s) {
            let path = format!("{}/fit_width_{:.0e}.svg", plot_dir, width * 10.0);
            let _ = plot_fit(&r, &path);
            md.push_str(&format!(
                "**width={}** — max err: {:.0} bps\n\n![](plots/fit_width_{:.0e}.svg)\n\n",
                width,
                r.max_iv_err * 10000.0,
                width * 10.0
            ));
        }
    }

    // ── Section 3: k* sensitivity ──────────────────────────────

    md.push_str("## 3. k\\* (ATM Log-Moneyness Offset)\n\n");
    md.push_str("Fixed: slope=0.6, width=0.5\n\n");
    md.push_str("| T | k\\* | max IV err (bps) | RMSE IV (bps) | theta | rho |\n");
    md.push_str("|---|-----|-----------------|--------------|-------|-----|\n");

    for &t in &t_vals {
        for &kstar in &kstar_vals {
            let s = Scenario {
                label: format!("T={}_k={}", t, kstar),
                t_expiry: t,
                slope: 0.6,
                width: 0.5,
                k_star: kstar,
                atm_vol,
            };
            if let Some(r) = run_scenario(s) {
                md.push_str(&format!(
                    "| {:.0e} | {:.3} | {:.1} | {:.1} | {:.4e} | {:.3} |\n",
                    r.scenario.t_expiry,
                    r.scenario.k_star,
                    r.max_iv_err * 10000.0,
                    r.rmse_iv * 10000.0,
                    r.theta,
                    r.rho,
                ));
            }
        }
    }

    // ── Section 4: Extreme case from user's question ───────────

    md.push_str("\n## 4. User Scenario: ATM=0.4, Wing=0.7 at k=-0.7\n\n");
    md.push_str("Steep skew (slope ~0.43/unit k), various T and k\\*\n\n");
    md.push_str("| T | k\\* | max IV err (bps) | RMSE IV (bps) | rho | converged |\n");
    md.push_str("|---|-----|-----------------|--------------|-----|----------|\n");

    for &t in &t_vals {
        for &kstar in &[0.0, 0.005, 0.01, -0.01] {
            // slope = (0.7 - 0.4) / 0.7 ≈ 0.429
            let s = Scenario {
                label: format!("user_T={}_k={}", t, kstar),
                t_expiry: t,
                slope: 0.429,
                width: 0.7,
                k_star: kstar,
                atm_vol,
            };
            if let Some(r) = run_scenario(s) {
                md.push_str(&format!(
                    "| {:.0e} | {:.3} | {:.1} | {:.1} | {:.3} | {} |\n",
                    r.scenario.t_expiry,
                    r.scenario.k_star,
                    r.max_iv_err * 10000.0,
                    r.rmse_iv * 10000.0,
                    r.rho,
                    r.converged
                ));

                // Plot for T=0.01
                if (t - 0.01).abs() < 1e-6 {
                    let kname = format!("{:.0e}", kstar.abs() * 1000.0);
                    let sign = if kstar >= 0.0 { "p" } else { "n" };
                    let path = format!("{}/fit_user_k{}_{}.svg", plot_dir, sign, kname);
                    let _ = plot_fit(&r, &path);
                    md.push_str(&format!(
                        "\n![T=0.01, k*={}](plots/fit_user_k{}_{}.svg)\n\n",
                        kstar, sign, kname
                    ));
                }
            }
        }
    }

    // ── Write report ────────────────────────────────────────────

    let report_path = "documents/fit_quality_report.md";
    let mut file = fs::File::create(report_path).expect("create report file");
    file.write_all(md.as_bytes()).expect("write report");

    println!("Report written to {}", report_path);
    println!("Plots written to {}/", plot_dir);
}
