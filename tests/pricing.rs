/// Black-76 pricing and greeks integration tests.
use essvi::pricing::black76;
use essvi::pricing::error::PricingError;

// ── Price tests ───────────────────────────────────────────────────────────────

#[test]
fn atm_call_price() {
    // ATM call: F=K=100, sigma=20%, T=1
    let c = black76::price(100.0, 100.0, 0.20, 1.0, 1).unwrap();
    // Reference: scipy.stats Black-76 ATM call ~ 7.965567455405804
    assert!(
        (c - 7.965567455405804).abs() < 1e-10,
        "ATM call price = {}, expected ~7.9656",
        c
    );
}

#[test]
fn atm_put_price() {
    // ATM put should equal ATM call (put-call parity: C - P = F - K = 0 at ATM)
    let c = black76::price(100.0, 100.0, 0.20, 1.0, 1).unwrap();
    let p = black76::price(100.0, 100.0, 0.20, 1.0, -1).unwrap();
    assert!(
        (c - p).abs() < 1e-14,
        "ATM put-call parity violated: C={}, P={}",
        c,
        p
    );
}

#[test]
fn put_call_parity() {
    // For any strike: C - P = F - K (undiscounted Black-76)
    let forward = 100.0;
    let sigma = 0.25;
    let t = 0.5;
    for &strike in &[80.0, 90.0, 100.0, 110.0, 120.0] {
        let c = black76::price(forward, strike, sigma, t, 1).unwrap();
        let p = black76::price(forward, strike, sigma, t, -1).unwrap();
        let parity_diff = (c - p) - (forward - strike);
        assert!(
            parity_diff.abs() < 1e-12,
            "Put-call parity violated at K={}: C-P={}, F-K={}",
            strike,
            c - p,
            forward - strike
        );
    }
}

#[test]
fn deep_itm_call() {
    // Deep ITM call: price ~ F - K
    let c = black76::price(100.0, 50.0, 0.20, 1.0, 1).unwrap();
    assert!(
        (c - 50.0).abs() < 0.01,
        "Deep ITM call should be ~50, got {}",
        c
    );
}

#[test]
fn deep_otm_call() {
    // Deep OTM call: price ~ 0
    let c = black76::price(100.0, 200.0, 0.10, 0.25, 1).unwrap();
    assert!(c < 1e-10, "Deep OTM call should be ~0, got {}", c);
}

#[test]
fn zero_vol_call() {
    // sigma=0: price = max(F-K, 0)
    let c = black76::price(100.0, 90.0, 0.0, 1.0, 1).unwrap();
    assert!(
        (c - 10.0).abs() < 1e-15,
        "Zero-vol ITM call should be 10, got {}",
        c
    );
    let c2 = black76::price(100.0, 110.0, 0.0, 1.0, 1).unwrap();
    assert!(
        c2.abs() < 1e-15,
        "Zero-vol OTM call should be 0, got {}",
        c2
    );
}

#[test]
fn zero_time_put() {
    // T=0: price = max(K-F, 0)
    let p = black76::price(100.0, 110.0, 0.20, 0.0, -1).unwrap();
    assert!(
        (p - 10.0).abs() < 1e-15,
        "Zero-time ITM put should be 10, got {}",
        p
    );
}

#[test]
fn price_positive_for_nonzero_vol() {
    // Any option with positive vol and time should have positive price
    for &q in &[1, -1] {
        let p = black76::price(100.0, 100.0, 0.01, 0.01, q).unwrap();
        assert!(
            p > 0.0,
            "Option price should be positive, got {} for q={}",
            p,
            q
        );
    }
}

// ── Delta tests ───────────────────────────────────────────────────────────────

#[test]
fn atm_call_delta() {
    // ATM call delta ~ 0.54 (slightly above 0.5 due to lognormal skew)
    let d = black76::delta(100.0, 100.0, 0.20, 1.0, 1).unwrap();
    assert!(
        (d - 0.5398278372770290).abs() < 1e-10,
        "ATM call delta = {}, expected ~0.5398",
        d
    );
}

#[test]
fn call_put_delta_relationship() {
    // Delta_put = Delta_call - 1
    let dc = black76::delta(100.0, 90.0, 0.25, 0.5, 1).unwrap();
    let dp = black76::delta(100.0, 90.0, 0.25, 0.5, -1).unwrap();
    assert!(
        (dp - (dc - 1.0)).abs() < 1e-14,
        "Delta_put != Delta_call - 1: dp={}, dc-1={}",
        dp,
        dc - 1.0
    );
}

#[test]
fn call_delta_bounded() {
    // Call delta should be in [0, 1]
    for &strike in &[50.0, 80.0, 100.0, 120.0, 150.0] {
        let d = black76::delta(100.0, strike, 0.30, 1.0, 1).unwrap();
        assert!(
            d >= 0.0 && d <= 1.0,
            "Call delta out of [0,1] at K={}: delta={}",
            strike,
            d
        );
    }
}

#[test]
fn put_delta_bounded() {
    // Put delta should be in [-1, 0]
    for &strike in &[50.0, 80.0, 100.0, 120.0, 150.0] {
        let d = black76::delta(100.0, strike, 0.30, 1.0, -1).unwrap();
        assert!(
            d >= -1.0 && d <= 0.0,
            "Put delta out of [-1,0] at K={}: delta={}",
            strike,
            d
        );
    }
}

// ── Gamma tests ───────────────────────────────────────────────────────────────

#[test]
fn gamma_positive() {
    let g = black76::gamma(100.0, 100.0, 0.20, 1.0).unwrap();
    assert!(g > 0.0, "Gamma should be positive, got {}", g);
}

#[test]
fn gamma_maximized_atm() {
    // Gamma is highest at ATM
    let g_atm = black76::gamma(100.0, 100.0, 0.20, 1.0).unwrap();
    let g_otm = black76::gamma(100.0, 120.0, 0.20, 1.0).unwrap();
    let g_itm = black76::gamma(100.0, 80.0, 0.20, 1.0).unwrap();
    assert!(g_atm > g_otm, "Gamma should be highest at ATM");
    assert!(g_atm > g_itm, "Gamma should be highest at ATM");
}

#[test]
fn gamma_numerical_check() {
    // Verify gamma matches numerical derivative of delta
    let f = 100.0;
    let k = 100.0;
    let sigma = 0.20;
    let t = 1.0;
    let eps = 1e-6;
    let d_up = black76::delta(f + eps, k, sigma, t, 1).unwrap();
    let d_dn = black76::delta(f - eps, k, sigma, t, 1).unwrap();
    let gamma_num = (d_up - d_dn) / (2.0 * eps);
    let gamma_exact = black76::gamma(f, k, sigma, t).unwrap();
    assert!(
        (gamma_exact - gamma_num).abs() < 1e-6,
        "Gamma mismatch: exact={}, numerical={}",
        gamma_exact,
        gamma_num
    );
}

// ── Vega tests ────────────────────────────────────────────────────────────────

#[test]
fn vega_positive() {
    let v = black76::vega(100.0, 100.0, 0.20, 1.0).unwrap();
    assert!(v > 0.0, "Vega should be positive, got {}", v);
}

#[test]
fn vega_numerical_check() {
    // Verify vega matches numerical derivative of price w.r.t. sigma
    let f = 100.0;
    let k = 100.0;
    let sigma = 0.20;
    let t = 1.0;
    let eps = 1e-6;
    let p_up = black76::price(f, k, sigma + eps, t, 1).unwrap();
    let p_dn = black76::price(f, k, sigma - eps, t, 1).unwrap();
    let vega_num = (p_up - p_dn) / (2.0 * eps);
    let vega_exact = black76::vega(f, k, sigma, t).unwrap();
    assert!(
        (vega_exact - vega_num).abs() < 1e-4,
        "Vega mismatch: exact={}, numerical={}",
        vega_exact,
        vega_num
    );
}

// ── Theta tests ───────────────────────────────────────────────────────────────

#[test]
fn theta_negative() {
    // Theta should be negative (time decay)
    let th = black76::theta(100.0, 100.0, 0.20, 1.0).unwrap();
    assert!(th < 0.0, "Theta should be negative, got {}", th);
}

#[test]
fn theta_numerical_check() {
    // Verify theta matches numerical derivative of price w.r.t. T
    let f = 100.0;
    let k = 100.0;
    let sigma = 0.20;
    let t = 1.0;
    let eps = 1e-6;
    let p_up = black76::price(f, k, sigma, t + eps, 1).unwrap();
    let p_dn = black76::price(f, k, sigma, t - eps, 1).unwrap();
    // dPrice/dT (positive since more time = more value)
    let dp_dt = (p_up - p_dn) / (2.0 * eps);
    // theta = -dPrice/dt = dPrice/dT (since we define theta w.r.t. time-to-expiry)
    // Actually our theta is dPrice/dT with a negative sign convention
    let theta_exact = black76::theta(f, k, sigma, t).unwrap();
    // theta_exact should be approximately -dp_dt... wait, theta IS dPrice/dT here
    // but with a negative sign: theta = -F*n(d1)*sigma/(2*sqrt(T))
    // dp_dt = F*n(d1)*sigma/(2*sqrt(T)) + correction terms
    // The leading term is what we compute, so it should be close
    assert!(
        (theta_exact + dp_dt).abs() < 0.01,
        "Theta mismatch: exact={}, -numerical_dPdT={}",
        theta_exact,
        -dp_dt
    );
}

// ── Discounted price tests ────────────────────────────────────────────────────

#[test]
fn discounted_price_basic() {
    let df = (-0.05_f64 * 1.0).exp();
    let p = black76::discounted_price(100.0, 100.0, 0.20, 1.0, 1, df).unwrap();
    let undiscounted = black76::price(100.0, 100.0, 0.20, 1.0, 1).unwrap();
    assert!(
        (p - df * undiscounted).abs() < 1e-14,
        "Discounted price should be df * undiscounted"
    );
}

#[test]
fn discounted_price_df_one() {
    // df=1.0 should give same as undiscounted
    let p = black76::discounted_price(100.0, 100.0, 0.20, 1.0, 1, 1.0).unwrap();
    let undiscounted = black76::price(100.0, 100.0, 0.20, 1.0, 1).unwrap();
    assert!(
        (p - undiscounted).abs() < 1e-15,
        "df=1.0 should equal undiscounted"
    );
}

// ── Combined greeks tests ─────────────────────────────────────────────────────

#[test]
fn greeks_consistency() {
    // Combined greeks should match individual greeks
    let f = 100.0;
    let k = 105.0;
    let sigma = 0.25;
    let t = 0.5;
    let q = 1;

    let g = black76::greeks(f, k, sigma, t, q).unwrap();
    let p = black76::price(f, k, sigma, t, q).unwrap();
    let d = black76::delta(f, k, sigma, t, q).unwrap();
    let gm = black76::gamma(f, k, sigma, t).unwrap();
    let v = black76::vega(f, k, sigma, t).unwrap();
    let th = black76::theta(f, k, sigma, t).unwrap();

    assert!(
        (g.price - p).abs() < 1e-15,
        "greeks.price != price: {} vs {}",
        g.price,
        p
    );
    assert!(
        (g.delta - d).abs() < 1e-15,
        "greeks.delta != delta: {} vs {}",
        g.delta,
        d
    );
    assert!(
        (g.gamma - gm).abs() < 1e-15,
        "greeks.gamma != gamma: {} vs {}",
        g.gamma,
        gm
    );
    assert!(
        (g.vega - v).abs() < 1e-15,
        "greeks.vega != vega: {} vs {}",
        g.vega,
        v
    );
    assert!(
        (g.theta - th).abs() < 1e-15,
        "greeks.theta != theta: {} vs {}",
        g.theta,
        th
    );
}

#[test]
fn greeks_put_consistency() {
    // Same for puts
    let g = black76::greeks(100.0, 95.0, 0.30, 0.25, -1).unwrap();
    let p = black76::price(100.0, 95.0, 0.30, 0.25, -1).unwrap();
    let d = black76::delta(100.0, 95.0, 0.30, 0.25, -1).unwrap();
    assert!((g.price - p).abs() < 1e-15);
    assert!((g.delta - d).abs() < 1e-15);
}

// ── Error handling tests ──────────────────────────────────────────────────────

#[test]
fn error_negative_forward() {
    let result = black76::price(-100.0, 100.0, 0.20, 1.0, 1);
    assert!(matches!(result, Err(PricingError::InvalidInput(_))));
}

#[test]
fn error_negative_strike() {
    let result = black76::price(100.0, -100.0, 0.20, 1.0, 1);
    assert!(matches!(result, Err(PricingError::InvalidInput(_))));
}

#[test]
fn error_negative_sigma() {
    let result = black76::price(100.0, 100.0, -0.20, 1.0, 1);
    assert!(matches!(result, Err(PricingError::InvalidInput(_))));
}

#[test]
fn error_negative_time() {
    let result = black76::price(100.0, 100.0, 0.20, -1.0, 1);
    assert!(matches!(result, Err(PricingError::InvalidInput(_))));
}

#[test]
fn error_invalid_q() {
    let result = black76::price(100.0, 100.0, 0.20, 1.0, 0);
    assert!(matches!(result, Err(PricingError::InvalidInput(_))));
    let result2 = black76::price(100.0, 100.0, 0.20, 1.0, 2);
    assert!(matches!(result2, Err(PricingError::InvalidInput(_))));
}

#[test]
fn error_invalid_discount_factor() {
    let result = black76::discounted_price(100.0, 100.0, 0.20, 1.0, 1, -0.1);
    assert!(matches!(result, Err(PricingError::InvalidInput(_))));
    let result2 = black76::discounted_price(100.0, 100.0, 0.20, 1.0, 1, 1.5);
    assert!(matches!(result2, Err(PricingError::InvalidInput(_))));
}

#[test]
fn error_display() {
    let e1 = PricingError::AboveMaximum {
        price: 105.0,
        maximum: 100.0,
    };
    let s1 = format!("{}", e1);
    assert!(s1.contains("exceeds theoretical maximum"));

    let e2 = PricingError::BelowIntrinsic {
        price: -1.0,
        intrinsic: 0.0,
    };
    let s2 = format!("{}", e2);
    assert!(s2.contains("below intrinsic"));

    let e3 = PricingError::InvalidInput("test".into());
    let s3 = format!("{}", e3);
    assert!(s3.contains("invalid input: test"));
}

#[test]
fn error_is_std_error() {
    // Verify PricingError implements std::error::Error
    let e: Box<dyn std::error::Error> = Box::new(PricingError::InvalidInput("test".into()));
    assert!(e.to_string().contains("test"));
}
