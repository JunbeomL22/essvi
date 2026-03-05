use criterion::{black_box, criterion_group, criterion_main, Criterion};
use essvi::calibration::{calibrate, solve_theta, CalibrationInput};
use essvi::nelder_mead::NelderMeadConfig;
use essvi::ssvi;

fn make_20pt_slice() -> (Vec<f64>, Vec<f64>, f64, f64) {
    let eta = 0.8;
    let gamma = 0.4;
    let rho = -0.35;
    let theta_star = 0.04;
    let k_star = -0.01;

    let theta = solve_theta(eta, gamma, rho, theta_star, k_star).unwrap();
    let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
    let w_market = ssvi::total_variance_slice(&k_slice, theta, eta, gamma, rho);

    (k_slice, w_market, theta_star, k_star)
}

fn bench_calibrate_20pt(c: &mut Criterion) {
    let (k_slice, w_market, theta_star, k_star) = make_20pt_slice();
    let input = CalibrationInput {
        k_slice: &k_slice,
        w_market: &w_market,
        theta_star,
        k_star,
    };
    let config = NelderMeadConfig::default();

    c.bench_function("calibrate_20pt_slice", |b| {
        b.iter(|| calibrate(black_box(&input), black_box(&config)))
    });
}

fn bench_solve_theta(c: &mut Criterion) {
    c.bench_function("solve_theta", |b| {
        b.iter(|| solve_theta(black_box(0.8), black_box(0.4), black_box(-0.35), black_box(0.04), black_box(0.0)))
    });
}

fn bench_total_variance_slice(c: &mut Criterion) {
    let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
    c.bench_function("total_variance_20pt", |b| {
        b.iter(|| ssvi::total_variance_slice(black_box(&k_slice), black_box(0.04), black_box(0.8), black_box(0.4), black_box(-0.35)))
    });
}

criterion_group!(benches, bench_calibrate_20pt, bench_solve_theta, bench_total_variance_slice);
criterion_main!(benches);
