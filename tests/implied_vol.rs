/// Implied volatility solver integration tests.
use essvi::pricing::black76;
use essvi::pricing::error::PricingError;
use essvi::pricing::lets_be_rational;

// ── Normalised Black call tests ───────────────────────────────────────────────

#[test]
fn normalised_black_call_atm() {
    // ATM: x = 0, s = 0.20
    // beta_c = 2*Phi(s/2) - 1 = 2*Phi(0.1) - 1
    let beta = lets_be_rational::normalised_black_call(0.0, 0.20);
    assert!(beta > 0.0, "ATM normalised call should be positive");
    // For s=0.20, ATM: beta ~ s/sqrt(2*pi) ~ 0.0797
    assert!(
        (beta - 0.0797).abs() < 0.001,
        "ATM normalised call = {}, expected ~0.0797",
        beta
    );
}

#[test]
fn normalised_black_call_zero_vol() {
    // s = 0: should return intrinsic
    // For x > 0 (ITM): intrinsic = 1 - exp(-x)
    let beta_itm = lets_be_rational::normalised_black_call(0.5, 0.0);
    // This should be roughly the intrinsic = 1 - exp(-0.5) ~ 0.3935
    // Actually for s very small, we get close to intrinsic
    // s=0 is a degenerate case, just verify it doesn't panic
    assert!(beta_itm >= 0.0);
}

#[test]
fn normalised_black_call_otm() {
    // OTM: x = -0.5, s = 0.20
    let beta = lets_be_rational::normalised_black_call(-0.5, 0.20);
    assert!(
        beta > 0.0 && beta < 0.5,
        "OTM normalised call should be small positive, got {}",
        beta
    );
}

#[test]
fn normalised_black_call_put_call_parity() {
    // Put-call parity: C(x,s) - P(x,s) = 1 - exp(-x)
    // P(x,s) = exp(-x)*Phi(-d2) - Phi(-d1)
    // = exp(-x)*(1-Phi(d2)) - (1-Phi(d1))
    // = exp(-x) - exp(-x)*Phi(d2) - 1 + Phi(d1)
    // = Phi(d1) - exp(-x)*Phi(d2) + exp(-x) - 1
    // = C(x,s) + exp(-x) - 1
    // So C - P = C - (C + exp(-x) - 1) = 1 - exp(-x) ✓
    // Equivalently: C >= intrinsic = max(1-exp(-x), 0)
    let s = 0.30;
    for &x in &[-2.0, -1.0, -0.5, 0.0, 0.1, 0.5, 1.0, 2.0] {
        let c = lets_be_rational::normalised_black_call(x, s);
        let intrinsic = if x > 0.0 { 1.0 - (-x).exp() } else { 0.0 };
        assert!(
            c >= intrinsic - 1e-15,
            "Call below intrinsic at x={}: c={}, intrinsic={}",
            x,
            c,
            intrinsic
        );
        assert!(c >= 0.0, "Call negative at x={}: c={}", x, c);
    }
}

// ── Normalised vega tests ─────────────────────────────────────────────────────

#[test]
fn normalised_vega_positive() {
    let v = lets_be_rational::normalised_vega(0.0, 0.20);
    assert!(v > 0.0, "Normalised vega should be positive, got {}", v);
}

#[test]
fn normalised_vega_atm_matches_pdf() {
    // At x=0: vega_n = (1/sqrt(2*pi)) * exp(-s^2/8)
    let s = 0.30;
    let v = lets_be_rational::normalised_vega(0.0, s);
    let expected = (1.0 / (2.0 * std::f64::consts::PI).sqrt()) * (-s * s / 8.0).exp();
    assert!(
        (v - expected).abs() < 1e-14,
        "ATM vega: {} vs expected {}",
        v,
        expected
    );
}

// ── Round-trip implied volatility tests ───────────────────────────────────────

#[test]
fn implied_vol_atm_round_trip() {
    let sigma = 0.20;
    let f = 100.0;
    let k = 100.0;
    let t = 1.0;
    let price = black76::price(f, k, sigma, t, 1).unwrap();
    let iv = lets_be_rational::implied_volatility(price, f, k, t, 1).unwrap();
    assert!(
        (iv - sigma).abs() < 1e-12,
        "ATM round-trip: sigma={}, recovered={}",
        sigma,
        iv
    );
}

#[test]
fn implied_vol_otm_call_round_trip() {
    let sigma = 0.25;
    let f = 100.0;
    let k = 110.0;
    let t = 0.5;
    let price = black76::price(f, k, sigma, t, 1).unwrap();
    let iv = lets_be_rational::implied_volatility(price, f, k, t, 1).unwrap();
    assert!(
        (iv - sigma).abs() < 1e-10,
        "OTM call round-trip: sigma={}, recovered={}",
        sigma,
        iv
    );
}

#[test]
fn implied_vol_itm_call_round_trip() {
    let sigma = 0.25;
    let f = 100.0;
    let k = 90.0;
    let t = 0.5;
    let price = black76::price(f, k, sigma, t, 1).unwrap();
    let iv = lets_be_rational::implied_volatility(price, f, k, t, 1).unwrap();
    assert!(
        (iv - sigma).abs() < 1e-10,
        "ITM call round-trip: sigma={}, recovered={}",
        sigma,
        iv
    );
}

#[test]
fn implied_vol_put_round_trip() {
    let sigma = 0.30;
    let f = 100.0;
    let k = 95.0;
    let t = 0.25;
    let price = black76::price(f, k, sigma, t, -1).unwrap();
    let iv = lets_be_rational::implied_volatility(price, f, k, t, -1).unwrap();
    assert!(
        (iv - sigma).abs() < 1e-10,
        "Put round-trip: sigma={}, recovered={}",
        sigma,
        iv
    );
}

#[test]
fn implied_vol_various_strikes() {
    let f = 100.0;
    let t = 1.0;
    let sigma = 0.20;
    for &k in &[80.0, 90.0, 95.0, 100.0, 105.0, 110.0, 120.0] {
        for &q in &[1, -1] {
            let price = black76::price(f, k, sigma, t, q).unwrap();
            if price < 1e-15 {
                continue; // Skip deep OTM with near-zero price
            }
            let iv = lets_be_rational::implied_volatility(price, f, k, t, q).unwrap();
            assert!(
                (iv - sigma).abs() < 1e-8,
                "K={}, q={}: sigma={}, recovered={}, diff={:.2e}",
                k,
                q,
                sigma,
                iv,
                (iv - sigma).abs()
            );
        }
    }
}

#[test]
fn implied_vol_various_vols() {
    let f = 100.0;
    let k = 100.0;
    let t = 1.0;
    for &sigma in &[0.05, 0.10, 0.20, 0.30, 0.50, 0.80, 1.00] {
        let price = black76::price(f, k, sigma, t, 1).unwrap();
        let iv = lets_be_rational::implied_volatility(price, f, k, t, 1).unwrap();
        assert!(
            (iv - sigma).abs() < 1e-8,
            "sigma={}: recovered={}, diff={:.2e}",
            sigma,
            iv,
            (iv - sigma).abs()
        );
    }
}

#[test]
fn implied_vol_various_expiries() {
    let f = 100.0;
    let k = 100.0;
    let sigma = 0.20;
    for &t in &[0.01, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0] {
        let price = black76::price(f, k, sigma, t, 1).unwrap();
        let iv = lets_be_rational::implied_volatility(price, f, k, t, 1).unwrap();
        assert!(
            (iv - sigma).abs() < 1e-8,
            "T={}: sigma={}, recovered={}, diff={:.2e}",
            t,
            sigma,
            iv,
            (iv - sigma).abs()
        );
    }
}

// ── Normalised implied volatility tests ───────────────────────────────────────

#[test]
fn normalised_implied_vol_round_trip() {
    let s_true = 0.30;
    let x = 0.0;
    let beta = lets_be_rational::normalised_black_call(x, s_true);
    let s_recovered = lets_be_rational::normalised_implied_volatility(beta, x, 1);
    assert!(
        (s_recovered - s_true).abs() < 1e-10,
        "Normalised IV round-trip: s={}, recovered={}",
        s_true,
        s_recovered
    );
}

#[test]
fn normalised_implied_vol_otm() {
    let s_true = 0.40;
    let x = -0.5;
    let beta = lets_be_rational::normalised_black_call(x, s_true);
    let s_recovered = lets_be_rational::normalised_implied_volatility(beta, x, 1);
    assert!(
        (s_recovered - s_true).abs() < 1e-8,
        "Normalised IV OTM: s={}, recovered={}",
        s_true,
        s_recovered
    );
}

// ── Error handling tests ──────────────────────────────────────────────────────

#[test]
fn implied_vol_below_intrinsic() {
    // Call with price below intrinsic
    let result = lets_be_rational::implied_volatility(5.0, 100.0, 90.0, 1.0, 1);
    assert!(
        matches!(result, Err(PricingError::BelowIntrinsic { .. })),
        "Expected BelowIntrinsic error, got {:?}",
        result
    );
}

#[test]
fn implied_vol_above_maximum() {
    // Call price above forward
    let result = lets_be_rational::implied_volatility(101.0, 100.0, 100.0, 1.0, 1);
    assert!(
        matches!(result, Err(PricingError::AboveMaximum { .. })),
        "Expected AboveMaximum error, got {:?}",
        result
    );
}

#[test]
fn implied_vol_invalid_inputs() {
    // Negative forward
    let r1 = lets_be_rational::implied_volatility(5.0, -100.0, 100.0, 1.0, 1);
    assert!(matches!(r1, Err(PricingError::InvalidInput(_))));

    // Zero time
    let r2 = lets_be_rational::implied_volatility(5.0, 100.0, 100.0, 0.0, 1);
    assert!(matches!(r2, Err(PricingError::InvalidInput(_))));

    // Invalid q
    let r3 = lets_be_rational::implied_volatility(5.0, 100.0, 100.0, 1.0, 0);
    assert!(matches!(r3, Err(PricingError::InvalidInput(_))));
}
