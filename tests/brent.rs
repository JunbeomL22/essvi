/// Brent root finder unit tests (migrated from src/solver/brent.rs).

use essvi::solver::brent::brent;

#[test]
fn find_sqrt2() {
    let res = brent(|x| x * x - 2.0, 1.0, 2.0, 1e-14, 100);
    assert!(res.converged);
    assert!((res.root - 2.0_f64.sqrt()).abs() < 1e-12);
}

#[test]
fn no_sign_change() {
    let res = brent(|x| x * x + 1.0, 0.0, 2.0, 1e-14, 100);
    assert!(!res.converged);
}
