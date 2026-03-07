use criterion::{black_box, criterion_group, criterion_main, Criterion};
use essvi::calibration::{
    calibrate, calibrate_with_calendar_penalty, solve_theta, CalibrationConfig, CalibrationInput,
    PrevSlice,
};
use essvi::ssvi;

fn make_20pt_slice() -> (Vec<f64>, Vec<f64>, f64, f64) {
    let eta = 0.8;
    let gamma = 0.4;
    let rho = -0.35;
    let theta_star = 0.04;
    let k_star = -0.01;

    let config = CalibrationConfig::default();
    let theta = solve_theta(eta, gamma, rho, theta_star, k_star, &config).unwrap();
    let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
    let w_market = ssvi::total_variance_slice(&k_slice, theta, eta, gamma, rho);

    (k_slice, w_market, theta_star, k_star)
}

/// Build 12 slices mimicking the real-world surface data.
fn make_surface_slices() -> Vec<(Vec<f64>, Vec<f64>, f64, f64, f64)> {
    let config = CalibrationConfig::default();
    let params: Vec<(f64, f64, f64, f64)> = vec![
        // (t_expiry, eta, gamma, rho)
        (0.0301, 0.54, 0.29, -0.32),
        (0.1068, 0.57, 0.33, -0.31),
        (0.1936, 0.56, 0.38, -0.26),
        (0.2795, 0.53, 0.40, -0.26),
        (0.4376, 0.52, 0.42, -0.27),
        (0.7014, 0.57, 0.43, -0.23),
        (0.9507, 0.55, 0.45, -0.23),
        (1.0274, 0.51, 0.46, -0.22),
        (1.1988, 0.58, 0.42, -0.23),
        (1.4495, 0.52, 0.46, -0.23),
        (1.9476, 0.53, 0.48, -0.22),
        (2.9452, 0.53, 0.50, -0.24),
    ];

    params
        .iter()
        .map(|&(t, eta, gamma, rho)| {
            let atm_vol = 0.20 - 0.02 * t.sqrt();
            let theta_star = atm_vol * atm_vol * t;
            let k_star = 0.03;
            let theta = solve_theta(eta, gamma, rho, theta_star, k_star, &config).unwrap();
            let k_slice: Vec<f64> = (0..60).map(|i| -0.3 + (i as f64) * 0.6 / 59.0).collect();
            let w_market = ssvi::total_variance_slice(&k_slice, theta, eta, gamma, rho);
            (k_slice, w_market, theta_star, k_star, t)
        })
        .collect()
}

fn bench_calibrate_20pt(c: &mut Criterion) {
    let (k_slice, w_market, theta_star, k_star) = make_20pt_slice();
    let input = CalibrationInput {
        k_slice: &k_slice,
        w_market: &w_market,
        theta_star,
        k_star,
        weights: None,
    };
    let config = CalibrationConfig::default();

    c.bench_function("calibrate_20pt_slice", |b| {
        b.iter(|| calibrate(black_box(&input), black_box(&config)))
    });
}

fn bench_solve_theta(c: &mut Criterion) {
    let k_stars = [0.01, 0.02, 0.03, 0.04, 0.05, 0.1];
    let config = CalibrationConfig::default();
    let mut group = c.benchmark_group("solve_theta");
    for &k_star in &k_stars {
        group.bench_function(format!("k_star={:.2}", k_star), |b| {
            b.iter(|| {
                solve_theta(
                    black_box(0.8),
                    black_box(0.4),
                    black_box(-0.35),
                    black_box(0.04),
                    black_box(k_star),
                    black_box(&config),
                )
            })
        });
    }
    group.finish();
}

fn bench_total_variance_slice(c: &mut Criterion) {
    let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
    c.bench_function("total_variance_20pt", |b| {
        b.iter(|| {
            ssvi::total_variance_slice(
                black_box(&k_slice),
                black_box(0.04),
                black_box(0.8),
                black_box(0.4),
                black_box(-0.35),
            )
        })
    });
}

fn bench_surface_calibration(c: &mut Criterion) {
    let slices = make_surface_slices();
    let config = CalibrationConfig::default();
    let k_penalty: Vec<f64> = (0..=48).map(|i| -0.8 + i as f64 * 0.025).collect();
    let lambda = 100.0;

    c.bench_function("surface_12_slices", |b| {
        b.iter(|| {
            let mut prev: Option<PrevSlice> = None;

            for (k_slice, w_market, theta_star, k_star, _t) in &slices {
                let weights: Vec<f64> = k_slice
                    .iter()
                    .map(|&k| if k >= -0.2 && k <= 0.2 { 3.0 } else { 1.0 })
                    .collect();

                let input = CalibrationInput {
                    k_slice: black_box(k_slice),
                    w_market: black_box(w_market),
                    theta_star: *theta_star,
                    k_star: *k_star,
                    weights: Some(&weights),
                };

                let res = match &prev {
                    None => calibrate(&input, &config).unwrap(),
                    Some(p) => {
                        // First do unconstrained to get initial guess
                        let unc = calibrate(&input, &config).unwrap();
                        let init = [unc.eta, unc.gamma, unc.rho];
                        calibrate_with_calendar_penalty(&input, &config, p, &k_penalty, lambda, &init)
                            .unwrap()
                    }
                };

                prev = Some(PrevSlice {
                    theta: res.theta,
                    eta: res.eta,
                    gamma: res.gamma,
                    rho: res.rho,
                });
            }
        })
    });
}

criterion_group!(
    benches,
    bench_calibrate_20pt,
    bench_solve_theta,
    bench_total_variance_slice,
    bench_surface_calibration
);
criterion_main!(benches);
