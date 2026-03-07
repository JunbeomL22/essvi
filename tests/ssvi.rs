/// SSVI model unit tests (migrated from src/model/ssvi.rs).
use essvi::model::ssvi::{no_arbitrage_satisfied, phi, total_variance};

#[test]
fn phi_basic() {
    // phi(1.0; eta=1.0, gamma=0.5) = 1.0 / (1.0^0.5 * 2.0^0.5) = 1/sqrt(2)
    let p = phi(1.0, 1.0, 0.5);
    assert!((p - 1.0 / 2.0_f64.sqrt()).abs() < 1e-12);
}

#[test]
fn atm_total_variance() {
    // At k=0: w(0,theta) = (theta/2)*{1 + sqrt(rho^2 + 1 - rho^2)} = (theta/2)*{1 + 1} = theta
    let w = total_variance(0.0, 0.04, 1.0, 0.5, -0.3);
    assert!((w - 0.04).abs() < 1e-12);
}

#[test]
fn no_arb() {
    assert!(no_arbitrage_satisfied(1.0, 0.5)); // 1.0 * 1.5 = 1.5 <= 2
    assert!(!no_arbitrage_satisfied(1.5, 0.5)); // 1.5 * 1.5 = 2.25 > 2
}
