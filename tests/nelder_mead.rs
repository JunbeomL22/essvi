/// Nelder-Mead optimizer unit tests (migrated from src/solver/nelder_mead.rs).

use essvi::solver::nelder_mead::{nelder_mead_bounded, NelderMeadConfig};

#[test]
fn rosenbrock_2d() {
    let res = nelder_mead_bounded(
        |x| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0] * x[0]).powi(2),
        &[-1.0, -1.0],
        &[-5.0, -5.0],
        &[5.0, 5.0],
        &NelderMeadConfig::default(),
    );
    assert!(res.converged);
    assert!((res.x[0] - 1.0).abs() < 1e-5);
    assert!((res.x[1] - 1.0).abs() < 1e-5);
}

#[test]
fn solution_on_boundary() {
    // min x^2 with x in [2, 5] => solution at x=2
    let res = nelder_mead_bounded(
        |x| x[0] * x[0],
        &[3.0],
        &[2.0],
        &[5.0],
        &NelderMeadConfig::default(),
    );
    assert!(res.converged);
    assert!((res.x[0] - 2.0).abs() < 1e-6);
}
