/// Math foundations tests: erf, erfc, erfcx, normal distributions, constants.
use essvi::math::constants;
use essvi::math::erf::{erf, erfc, erfcx};
use essvi::math::normal;
use essvi::math::normal_hp;

// ── Error function tests ────────────────────────────────────────────────────

#[test]
fn erf_at_zero() {
    assert!(erf(0.0).abs() < 1e-15);
}

#[test]
fn erf_reference_values() {
    // (x, erf(x)) pairs from Abramowitz & Stegun Table 7.1
    let cases: &[(f64, f64)] = &[
        (0.25, 2.76326390168236901e-01),
        (0.5, 5.20499877813046519e-01),
        (1.0, 8.42700792949714861e-01),
        (1.5, 9.66105146475310728e-01),
        (2.0, 9.95322265018952734e-01),
        (3.0, 9.99977909503001415e-01),
    ];
    for &(x, expected) in cases {
        let val = erf(x);
        let err = (val - expected).abs();
        assert!(
            err < 2e-15,
            "erf({}) = {:.18e}, expected {:.18e}, err = {:.2e}",
            x,
            val,
            expected,
            err
        );
    }
}

#[test]
fn erf_symmetry() {
    // erf(-x) = -erf(x)
    for &x in &[0.1, 0.5, 1.0, 2.0, 5.0] {
        let diff = (erf(-x) + erf(x)).abs();
        assert!(diff < 1e-15, "erf(-{}) + erf({}) = {}", x, x, diff);
    }
}

#[test]
fn erf_large_argument() {
    // erf(x) -> 1 for large x
    assert!((erf(6.0) - 1.0).abs() < 1e-15);
    assert!((erf(10.0) - 1.0).abs() < 1e-15);
}

#[test]
fn erfc_at_zero() {
    assert!((erfc(0.0) - 1.0).abs() < 1e-15);
}

#[test]
fn erfc_complement() {
    // erf(x) + erfc(x) = 1
    for &x in &[0.1, 0.5, 0.84, 1.0, 1.25, 2.0, 3.0, 5.0] {
        let sum = erf(x) + erfc(x);
        assert!(
            (sum - 1.0).abs() < 1e-14,
            "erf({}) + erfc({}) = {}",
            x,
            x,
            sum
        );
    }
}

#[test]
fn erfc_large_argument() {
    assert!(erfc(28.0) == 0.0);
    assert!(erfc(-6.0) == 2.0);
}

#[test]
fn erfcx_at_zero() {
    assert!((erfcx(0.0) - 1.0).abs() < 1e-15);
}

#[test]
fn erfcx_reference() {
    // erfcx(1) = exp(1) * erfc(1) ~ 0.42758357615580700
    let val = erfcx(1.0);
    assert!(
        (val - 0.42758357615580700).abs() < 1e-14,
        "erfcx(1.0) = {:.18e}",
        val
    );
}

#[test]
fn erfcx_large_argument_bounded() {
    // erfcx(x) ~ 1/(sqrt(pi)*x) for large x -- should not overflow
    let val = erfcx(100.0);
    assert!(val.is_finite(), "erfcx(100.0) overflowed");
    assert!(val > 0.0, "erfcx(100.0) should be positive");
}

// ── Standard normal distribution tests ──────────────────────────────────────

#[test]
fn norm_pdf_at_zero() {
    let val = normal::norm_pdf(0.0);
    let expected = constants::ONE_OVER_SQRT_TWO_PI;
    assert!(
        (val - expected).abs() < 1e-15,
        "norm_pdf(0) = {}, expected {}",
        val,
        expected
    );
}

#[test]
fn norm_pdf_symmetry() {
    for &x in &[0.5, 1.0, 2.0, 3.0] {
        let diff = (normal::norm_pdf(x) - normal::norm_pdf(-x)).abs();
        assert!(diff < 1e-15, "norm_pdf not symmetric at x={}", x);
    }
}

#[test]
fn norm_cdf_at_zero() {
    assert!((normal::norm_cdf(0.0) - 0.5).abs() < 1e-15);
}

#[test]
fn norm_cdf_reference_values() {
    let cases: &[(f64, f64)] = &[
        (1.0, 0.8413447460685429),
        (2.0, 0.9772498680518208),
        (3.0, 0.9986501019683699),
        (-1.0, 0.15865525393145702),
        (-2.0, 0.022750131948179195),
    ];
    for &(x, expected) in cases {
        let val = normal::norm_cdf(x);
        let err = (val - expected).abs();
        assert!(
            err < 1e-14,
            "norm_cdf({}) = {:.18e}, expected {:.18e}, err = {:.2e}",
            x,
            val,
            expected,
            err
        );
    }
}

#[test]
fn norm_cdf_monotone() {
    let mut prev = normal::norm_cdf(-5.0);
    for i in -49..50 {
        let x = i as f64 * 0.1;
        let val = normal::norm_cdf(x);
        assert!(val >= prev, "norm_cdf not monotone at x={}", x);
        prev = val;
    }
}

#[test]
fn norm_inv_cdf_at_half() {
    assert!(normal::norm_inv_cdf(0.5).abs() < 1e-12);
}

#[test]
fn norm_inv_cdf_round_trip() {
    // Phi^{-1}(Phi(x)) = x
    for &x in &[-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0] {
        let p = normal::norm_cdf(x);
        let recovered = normal::norm_inv_cdf(p);
        assert!(
            (recovered - x).abs() < 1e-8,
            "round-trip failed: x={}, Phi(x)={}, Phi^-1(Phi(x))={}",
            x,
            p,
            recovered
        );
    }
}

#[test]
fn norm_inv_cdf_boundaries() {
    assert_eq!(normal::norm_inv_cdf(0.0), f64::NEG_INFINITY);
    assert_eq!(normal::norm_inv_cdf(1.0), f64::INFINITY);
    assert!(normal::norm_inv_cdf(f64::NAN).is_nan());
}

// ── High-precision normal distribution tests ────────────────────────────────

#[test]
fn norm_hp_cdf_at_zero() {
    assert!((normal_hp::norm_cdf(0.0) - 0.5).abs() < 1e-15);
}

#[test]
fn norm_hp_cdf_matches_standard_in_normal_range() {
    for &x in &[-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0] {
        let std_val = normal::norm_cdf(x);
        let hp_val = normal_hp::norm_cdf(x);
        assert!(
            (std_val - hp_val).abs() < 1e-14,
            "HP and standard CDF differ at x={}: {} vs {}",
            x,
            hp_val,
            std_val
        );
    }
}

#[test]
fn norm_hp_cdf_extreme_tail_nonzero() {
    // High-precision CDF should not return 0 for extreme negative arguments
    for &x in &[-10.0, -20.0, -30.0, -37.0] {
        let val = normal_hp::norm_cdf(x);
        assert!(
            val > 0.0,
            "norm_hp_cdf({}) returned 0, should be positive",
            x
        );
    }
}

#[test]
fn norm_hp_cdf_asymptotic_decreasing() {
    // In the tail, CDF should be monotonically decreasing toward 0
    let mut prev = normal_hp::norm_cdf(-10.0);
    for &x in &[-15.0, -20.0, -25.0, -30.0] {
        let val = normal_hp::norm_cdf(x);
        assert!(
            val < prev,
            "norm_hp_cdf not decreasing: f({}) = {} >= f({:.0})",
            x,
            val,
            x + 5.0
        );
        assert!(val > 0.0);
        prev = val;
    }
}

#[test]
fn norm_hp_inv_cdf_at_half() {
    assert!(normal_hp::norm_inv_cdf(0.5).abs() < 1e-14);
}

#[test]
fn norm_hp_inv_cdf_round_trip() {
    for &x in &[-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0] {
        let p = normal_hp::norm_cdf(x);
        let recovered = normal_hp::norm_inv_cdf(p);
        assert!(
            (recovered - x).abs() < 1e-8,
            "HP round-trip failed: x={}, CDF(x)={}, inv_CDF={}",
            x,
            p,
            recovered
        );
    }
}

// ── Constants tests ─────────────────────────────────────────────────────────

#[test]
fn constants_machine_epsilon() {
    assert_eq!(constants::DBL_EPSILON, f64::EPSILON);
}

#[test]
fn constants_sqrt_two_pi() {
    let computed = (2.0 * std::f64::consts::PI).sqrt();
    assert!(
        (constants::SQRT_TWO_PI - computed).abs() < 1e-14,
        "SQRT_TWO_PI mismatch"
    );
}

#[test]
fn constants_one_over_sqrt_two_pi() {
    let computed = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
    assert!(
        (constants::ONE_OVER_SQRT_TWO_PI - computed).abs() < 1e-15,
        "ONE_OVER_SQRT_TWO_PI mismatch"
    );
}

#[test]
fn constants_sqrt_roots() {
    assert!((constants::SQRT_DBL_EPSILON - f64::EPSILON.sqrt()).abs() < 1e-16);
    assert!((constants::FOURTH_ROOT_DBL_EPSILON - f64::EPSILON.sqrt().sqrt()).abs() < 1e-10);
}
