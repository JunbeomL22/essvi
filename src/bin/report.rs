/// Generate SSVI fit quality report across parameter grid:
///   T (expiry), skew slope — with η diagnostics showing constraint saturation.
use essvi::calibration::{CalibrationConfig, CalibrationInput, calibrate};
use essvi::ssvi;
use plotters::prelude::*;
use std::fs;
use std::io::Write;

// ── Scenario parameters ─────────────────────────────────────

struct Scenario {
    label: String,
    t_expiry: f64,
    slope: f64, // put-side vol increase per unit |k|
    k_star: f64,
    atm_vol: f64,
}

fn make_market_data(s: &Scenario) -> (Vec<f64>, Vec<f64>) {
    // 30 points from -0.5 to +0.5
    let k_lo = -0.4;
    let k_hi = 0.4;
    let n = 30;
    let k_slice: Vec<f64> = (0..n)
        .map(|i| k_lo + (i as f64) * (k_hi - k_lo) / (n - 1) as f64)
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
                0.5 * s.slope * k
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
    no_arb_usage: f64, // eta * (1 + |rho|) — how close to 2.0
    converged: bool,
    max_iv_err: f64,
    rmse_iv: f64,
    k_slice: Vec<f64>,
    iv_market: Vec<f64>,
    iv_fit: Vec<f64>,
}

fn run_scenario(s: Scenario) -> Option<FitResult> {
    let (k_slice, w_market) = make_market_data(&s);
    let theta_star = s.atm_vol * s.atm_vol * s.t_expiry;

    let weights: Vec<f64> = k_slice
        .iter()
        .map(|&k| if k >= -0.2 && k <= 0.2 { 3.0 } else { 1.0 })
        .collect();

    let input = CalibrationInput {
        k_slice: &k_slice,
        w_market: &w_market,
        theta_star,
        k_star: s.k_star,
        weights: Some(&weights),
    };

    let config = CalibrationConfig::default();
    let res = calibrate(&input, &config).ok()?;

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
        phi: res.phi(),
        eta: res.eta,
        gamma: res.gamma,
        rho: res.rho,
        theta: res.theta,
        no_arb_usage: res.no_arb_usage(),
        converged: res.optimizer.converged,
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
        "T={}, slope={}, k*={}",
        result.scenario.t_expiry, result.scenario.slope, result.scenario.k_star
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
    chart
        .draw_series(
            result
                .k_slice
                .iter()
                .zip(result.iv_market.iter())
                .map(|(&k, &iv)| Circle::new((k, iv), 3, BLUE.filled())),
        )?
        .label("Market")
        .legend(|(x, y)| Circle::new((x + 10, y), 3, BLUE.filled()));

    // Fit line
    chart
        .draw_series(LineSeries::new(
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
        .x_desc("Slope")
        .y_desc("T")
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
                format!("{}", t_vals[i])
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

    // Parameter grid — T=1 uses milder slopes since SSVI saturates there
    let cases: Vec<(f64, f64)> = vec![
        (0.03, 0.5),
        (0.03, 1.0),
        (0.1, 0.5),
        (0.1, 1.0),
        (0.25, 0.5),
        (0.25, 1.0),
        (0.5, 0.5),
        (0.5, 1.0),
        (1.0, 0.2),
        (1.0, 0.4),
    ];
    let atm_vol = 0.4;
    let k_star = 0.01;

    let mut md = String::new();

    md.push_str("# SSVI Slice Fit Quality Report\n\n");
    md.push_str(&format!(
        "ATM vol = {}  |  k range = [-0.4, 0.4]  |  k\\* = {}\n\n",
        atm_vol, k_star
    ));

    // ── Section 1: T × Slope table ─────────────────────────────

    md.push_str("## 1. Expiry (T) vs Skew Slope\n\n");
    md.push_str("| T | Slope | max IV err (bps) | RMSE IV (bps) | eta | gamma | rho | phi | eta*(1+\\|rho\\|) | converged |\n");
    md.push_str("|---:|------:|----------------:|--------------:|----:|------:|----:|----:|---------------:|:---------:|\n");

    let mut grid_results: Vec<FitResult> = Vec::new();

    for &(t, slope) in &cases {
        let s = Scenario {
            label: format!("T={}_s={}", t, slope),
            t_expiry: t,
            slope,
            k_star,
            atm_vol,
        };
        if let Some(r) = run_scenario(s) {
            md.push_str(&format!(
                "| {} | {} | {:.1} | {:.1} | {:.3} | {:.3} | {:.3} | {:.2} | {:.3} | {} |\n",
                r.scenario.t_expiry,
                r.scenario.slope,
                r.max_iv_err * 10000.0,
                r.rmse_iv * 10000.0,
                r.eta,
                r.gamma,
                r.rho,
                r.phi,
                r.no_arb_usage,
                if r.converged { "yes" } else { "**no**" }
            ));
            grid_results.push(r);
        }
    }

    md.push_str("\n");

    // ── Section 2: Fit plots for all T × Slope ─────────────────

    md.push_str("## 2. Fit Plots (all combinations)\n\n");

    for &(t, slope) in &cases {
        let name = format!(
            "T{}_s{}",
            format!("{}", t).replace('.', "p"),
            format!("{}", slope).replace('.', "p")
        );
        let s = Scenario {
            label: name.clone(),
            t_expiry: t,
            slope,
            k_star,
            atm_vol,
        };
        if let Some(r) = run_scenario(s) {
            let path = format!("{}/fit_{}.svg", plot_dir, name);
            let _ = plot_fit(&r, &path);
            md.push_str(&format!(
                "**T={}, slope={}** — max err: {:.0} bps, eta={:.3}, rho={:.3}, eta*(1+|rho|)={:.3}\n\n",
                t, slope,
                r.max_iv_err * 10000.0,
                r.eta, r.rho, r.no_arb_usage
            ));
            md.push_str(&format!("![{}](plots/fit_{}.svg)\n\n", name, name));
        }
    }

    // ── Section 3: Constraint saturation analysis ────────────────

    md.push_str("## 3. No-Arbitrage Constraint Saturation\n\n");
    md.push_str("The SSVI no-arb condition `eta * (1 + |rho|) <= 2` limits how much skew the model can produce.\n");
    md.push_str("When this value approaches 2.0, the optimizer is constrained and fit quality degrades.\n\n");

    md.push_str("| T | Slope | eta | rho | eta*(1+\\|rho\\|) | saturated | max err (bps) |\n");
    md.push_str("|---:|------:|----:|----:|---------------:|:---------:|--------------:|\n");

    for r in &grid_results {
        let saturated = r.no_arb_usage > 1.95;
        md.push_str(&format!(
            "| {} | {} | {:.3} | {:.3} | {:.3} | {} | {:.0} |\n",
            r.scenario.t_expiry,
            r.scenario.slope,
            r.eta,
            r.rho,
            r.no_arb_usage,
            if saturated { "**YES**" } else { "no" },
            r.max_iv_err * 10000.0
        ));
    }

    // ── Write report ────────────────────────────────────────────

    let report_path = "documents/fit_quality_report.md";
    let mut file = fs::File::create(report_path).expect("create report file");
    file.write_all(md.as_bytes()).expect("write report");

    println!("Report written to {}", report_path);
    println!("Plots written to {}/", plot_dir);
}
