/// Shared types and helpers for fit_real and fit_real_surface binaries.
///
/// Contains the synthetic market data generators (SliceData, make_slice,
/// build_market_slices), the FitResult struct that both binaries populate,
/// and the plot_fit SVG renderer.

use crate::model::ssvi;
use plotters::prelude::*;

// ── Slice definition ────────────────────────────────────────

/// A single expiry slice of market-like implied volatility data.
pub struct SliceData {
    pub t_expiry: f64,
    pub k: Vec<f64>,
    pub iv: Vec<f64>, // implied volatility (decimal)
}

/// Build a single slice with a realistic vol smile shape.
///
/// Uses a hyperbolic form similar to SVI:
///   σ(k) = atm_vol + a * (rho_shape * k + sqrt(k² + d²)) + noise
///
/// This produces the characteristic skew + smile that SSVI-like models
/// fit well, matching real market data patterns.
pub fn make_slice(
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

    // Simple deterministic "noise" using a hash-like function
    let seed = (t_expiry * 10000.0) as u64;

    // SVI-like parameters derived from inputs
    let rho_shape = -0.5; // less negative = more call-side vol
    let d = 0.08 + 0.05 * t_expiry.sqrt(); // smoothing (wider for longer T)
    let a = skew * 0.45; // amplitude

    for i in 0..n {
        let frac = i as f64 / (n - 1) as f64;
        let k = k_lo + frac * (k_hi - k_lo);

        // Hyperbolic smile: a * (rho*k + sqrt(k² + d²))
        let hyp = a * (rho_shape * k + (k * k + d * d).sqrt());
        // Small quadratic correction for wing behavior
        let wing = curvature * 0.01 * k * k;

        // Deterministic perturbation to simulate market scatter
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

/// Generate ~60 data points per slice across 12 expiries, approximating
/// the scatter plots visible in real-data.png. The patterns follow typical
/// equity index vol surface characteristics at each expiry.
pub fn build_market_slices() -> Vec<SliceData> {
    vec![
        // T=0.0301: Very short expiry, narrow tradable range
        make_slice(0.0301, -0.30, 0.20, 60, 0.200, 0.28, 0.08, 0.0015),
        // T=0.1068
        make_slice(0.1068, -0.40, 0.25, 60, 0.185, 0.24, 0.06, 0.0012),
        // T=0.1936
        make_slice(0.1936, -0.45, 0.30, 60, 0.175, 0.22, 0.06, 0.0010),
        // T=0.2795
        make_slice(0.2795, -0.50, 0.35, 60, 0.170, 0.20, 0.05, 0.0010),
        // T=0.4376
        make_slice(0.4376, -0.55, 0.40, 60, 0.165, 0.18, 0.04, 0.0008),
        // T=0.7014
        make_slice(0.7014, -0.55, 0.45, 60, 0.158, 0.15, 0.04, 0.0008),
        // T=0.9507
        make_slice(0.9507, -0.55, 0.45, 60, 0.155, 0.14, 0.04, 0.0007),
        // T=1.0274
        make_slice(1.0274, -0.55, 0.45, 60, 0.153, 0.13, 0.04, 0.0007),
        // T=1.1988
        make_slice(1.1988, -0.60, 0.50, 60, 0.152, 0.12, 0.035, 0.0007),
        // T=1.4495
        make_slice(1.4495, -0.65, 0.55, 60, 0.155, 0.11, 0.03, 0.0006),
        // T=1.9476
        make_slice(1.9476, -0.75, 0.60, 60, 0.160, 0.10, 0.025, 0.0006),
        // T=2.9452
        make_slice(2.9452, -0.90, 0.65, 60, 0.168, 0.09, 0.02, 0.0005),
    ]
}

// ── Fit result ──────────────────────────────────────────────

/// Result of fitting a single SSVI slice to market data.
///
/// Used by both per-slice and surface-level fit binaries.
/// Calendar arbitrage fields default to zero for per-slice fits
/// that do not check calendar constraints.
pub struct FitResult {
    pub t_expiry: f64,
    pub eta: f64,
    pub gamma: f64,
    pub rho: f64,
    pub theta: f64,
    pub phi: f64,
    pub no_arb_usage: f64,
    pub converged: bool,
    pub max_iv_err_bps: f64,
    pub rmse_iv_bps: f64,
    pub avg_price_err_bps: f64,
    pub calendar_violations: usize,
    pub max_calendar_violation_bps: f64,
    pub k: Vec<f64>,
    pub iv_market: Vec<f64>,
    pub iv_fit: Vec<f64>,
}

// ── Plot ────────────────────────────────────────────────────

/// Render a market-vs-fit IV plot to an SVG file.
///
/// The caller supplies the plot `title`; this keeps the shared function
/// independent of per-binary formatting choices.
pub fn plot_fit(r: &FitResult, path: &str, title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(path, (640, 420)).into_drawing_area();
    root.fill(&WHITE)?;

    let k_min = *r.k.first().unwrap();
    let k_max = *r.k.last().unwrap();
    let all_iv: Vec<f64> = r.iv_market.iter().chain(r.iv_fit.iter()).copied().collect();
    let iv_min = all_iv.iter().cloned().fold(f64::INFINITY, f64::min) * 0.92;
    let iv_max = all_iv.iter().cloned().fold(0.0_f64, f64::max) * 1.08;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 13))
        .margin(10)
        .x_label_area_size(35)
        .y_label_area_size(55)
        .build_cartesian_2d(k_min..k_max, iv_min..iv_max)?;

    chart
        .configure_mesh()
        .x_desc("log-moneyness")
        .y_desc("implied volatility")
        .draw()?;

    // Market data points
    chart.draw_series(
        r.k.iter()
            .zip(r.iv_market.iter())
            .map(|(&k, &iv)| Circle::new((k, iv), 3, BLACK.filled())),
    )?
    .label("Market")
    .legend(|(x, y)| Circle::new((x + 10, y), 3, BLACK.filled()));

    // SSVI fit curve (dense)
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
