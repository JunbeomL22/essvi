/// Error function implementations: erf, erfc, and erfcx.
///
/// Uses rational Chebyshev approximations from the Sun Microsystems fdlibm
/// implementation (s_erf.c), providing machine-precision accuracy across the
/// full domain.

// ── Constants ───────────────────────────────────────────────────────────────

/// erf(1) truncated to float precision
const ERX: f64 = 8.45062911510467529297e-01;

// ── Region 1: |x| < 0.84375 ────────────────────────────────────────────────
// erf(x) = x + x * P(x^2)/Q(x^2)

const PP0: f64 = 1.28379167095512558561e-01;
const PP1: f64 = -3.25042107247001499370e-01;
const PP2: f64 = -2.84817495755985104766e-02;
const PP3: f64 = -5.77027029648944159157e-03;
const PP4: f64 = -2.37630166566501626084e-05;

const QQ1: f64 = 3.97917223959155352819e-01;
const QQ2: f64 = 6.50222499887672944485e-02;
const QQ3: f64 = 5.08130628187576562776e-03;
const QQ4: f64 = 1.32494738004321644526e-04;
const QQ5: f64 = -3.96022827877536812320e-06;

// ── Region 2: 0.84375 <= |x| < 1.25 ────────────────────────────────────────
// erf(x) = ERX + P(|x|-1)/Q(|x|-1)

const PA0: f64 = -2.36211856075265944077e-03;
const PA1: f64 = 4.14856118683748331666e-01;
const PA2: f64 = -3.72207876035701323847e-01;
const PA3: f64 = 3.18346619901161753674e-01;
const PA4: f64 = -1.10894694282396677476e-01;
const PA5: f64 = 3.54783043195201877747e-02;
const PA6: f64 = -2.16637559983254089680e-03;

const QA1: f64 = 1.06420880400844228286e-01;
const QA2: f64 = 5.40397917702171048937e-01;
const QA3: f64 = 7.18286544141962539399e-02;
const QA4: f64 = 1.26171219808761642112e-01;
const QA5: f64 = 1.36370839120290507362e-02;
const QA6: f64 = 1.19844998467991074170e-02;

// ── Region 3a: 1.25 <= |x| < 1/0.35 (~2.857) ──────────────────────────────

const RA0: f64 = -9.86494403484714822705e-03;
const RA1: f64 = -6.93858572707181764372e-01;
const RA2: f64 = -1.05586262253232909814e+01;
const RA3: f64 = -6.23753324503260060396e+01;
const RA4: f64 = -1.62396669462573071767e+02;
const RA5: f64 = -1.84605092906711035994e+02;
const RA6: f64 = -8.12874355063065934246e+01;
const RA7: f64 = -9.81432934416914548592e+00;

const SA1: f64 = 1.96512716674392571292e+01;
const SA2: f64 = 1.37657754143519702237e+02;
const SA3: f64 = 4.34565877475229228608e+02;
const SA4: f64 = 6.45387271733267880594e+02;
const SA5: f64 = 4.29008140027567833386e+02;
const SA6: f64 = 1.08635005541779435134e+02;
const SA7: f64 = 6.57024977031928170135e+00;
const SA8: f64 = -6.04244152148580987438e-02;

// ── Region 3b: |x| >= 1/0.35 (~2.857) ──────────────────────────────────────

const RB0: f64 = -9.86494292470009928597e-03;
const RB1: f64 = -7.99283237680523006574e-01;
const RB2: f64 = -1.77579549177547519889e+01;
const RB3: f64 = -1.60636384855557935030e+02;
const RB4: f64 = -6.37566443368389085394e+02;
const RB5: f64 = -1.02509513161107724954e+03;
const RB6: f64 = -4.83519191608651397019e+02;

const SB1: f64 = 3.03380607875625778203e+01;
const SB2: f64 = 3.25792512996573918826e+02;
const SB3: f64 = 1.53672958608443695994e+03;
const SB4: f64 = 3.19985821950859553908e+03;
const SB5: f64 = 2.55305040643316442583e+03;
const SB6: f64 = 4.74528541206955367215e+02;
const SB7: f64 = -2.24409524465858183362e+01;

/// Compute erf(x) using rational Chebyshev approximations.
///
/// Accurate to machine precision across the full domain.
///
/// # Examples
/// ```
/// # use essvi::math::erf::erf;
/// assert!(erf(0.0).abs() < 1e-15);
/// assert!((erf(1.0) - 0.8427007929497149).abs() < 1e-15);
/// ```
pub fn erf(x: f64) -> f64 {
    let ax = x.abs();
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };

    if ax < 0.84375 {
        if ax < 3.7252902984619140625e-09 {
            // |x| < 2^-28: erf(x) ~ x * (2/sqrt(pi))
            if ax < f64::MIN_POSITIVE {
                return x;
            }
            return x + x * PP0;
        }
        let z = x * x;
        let r = PP0 + z * (PP1 + z * (PP2 + z * (PP3 + z * PP4)));
        let s = 1.0 + z * (QQ1 + z * (QQ2 + z * (QQ3 + z * (QQ4 + z * QQ5))));
        return x + x * r / s;
    }

    if ax < 1.25 {
        let s = ax - 1.0;
        let p = PA0 + s * (PA1 + s * (PA2 + s * (PA3 + s * (PA4 + s * (PA5 + s * PA6)))));
        let q = 1.0 + s * (QA1 + s * (QA2 + s * (QA3 + s * (QA4 + s * (QA5 + s * QA6)))));
        return sign * (ERX + p / q);
    }

    if ax >= 6.0 {
        return sign;
    }

    let erfc_val = erfc_inner(ax);
    sign * (1.0 - erfc_val)
}

/// Compute erfc(x) = 1 - erf(x) using rational Chebyshev approximations.
///
/// More accurate than computing `1.0 - erf(x)` for large x, where erf(x) is
/// close to 1 and subtraction would cause catastrophic cancellation.
///
/// # Examples
/// ```
/// # use essvi::math::erf::erfc;
/// assert!((erfc(0.0) - 1.0).abs() < 1e-15);
/// assert!((erfc(5.0) - 1.5374597944280349e-12).abs() < 1e-22);
/// ```
pub fn erfc(x: f64) -> f64 {
    let ax = x.abs();

    if ax < 0.84375 {
        if ax < 3.7252902984619140625e-09 {
            return 1.0 - x;
        }
        let z = x * x;
        let r = PP0 + z * (PP1 + z * (PP2 + z * (PP3 + z * PP4)));
        let s = 1.0 + z * (QQ1 + z * (QQ2 + z * (QQ3 + z * (QQ4 + z * QQ5))));
        if ax < 0.25 {
            return 1.0 - (x + x * r / s);
        }
        return 0.5 - (x + x * r / s - 0.5);
    }

    if ax < 1.25 {
        let s = ax - 1.0;
        let p = PA0 + s * (PA1 + s * (PA2 + s * (PA3 + s * (PA4 + s * (PA5 + s * PA6)))));
        let q = 1.0 + s * (QA1 + s * (QA2 + s * (QA3 + s * (QA4 + s * (QA5 + s * QA6)))));
        if x >= 0.0 {
            return 1.0 - ERX - p / q;
        }
        return 1.0 + ERX + p / q;
    }

    if x >= 28.0 {
        return 0.0;
    }
    if x <= -6.0 {
        return 2.0;
    }

    let erfc_val = erfc_inner(ax);
    if x >= 0.0 { erfc_val } else { 2.0 - erfc_val }
}

/// Compute erfcx(x) = exp(x^2) * erfc(x), the scaled complementary error function.
///
/// Avoids overflow/underflow of computing exp(x^2) and erfc(x) separately.
/// erfcx(x) is bounded and well-behaved for all x >= 0.
///
/// # Examples
/// ```
/// # use essvi::math::erf::erfcx;
/// assert!((erfcx(0.0) - 1.0).abs() < 1e-15);
/// assert!((erfcx(1.0) - 0.42758357615580700).abs() < 1e-14);
/// ```
pub fn erfcx(x: f64) -> f64 {
    let ax = x.abs();

    if ax < 1.25 {
        return (x * x).exp() * erfc(x);
    }

    if ax >= 28.0 {
        if x >= 0.0 {
            return 0.5641895835477563 / ax;
        }
        return 2.0 * (ax * ax).exp();
    }

    // erfcx(|x|) = exp(-0.5625 + R/S) / |x|
    let s = 1.0 / (ax * ax);

    let r = if ax < 2.857142857142857 {
        let rr =
            RA0 + s * (RA1 + s * (RA2 + s * (RA3 + s * (RA4 + s * (RA5 + s * (RA6 + s * RA7))))));
        let ss = 1.0
            + s * (SA1
                + s * (SA2 + s * (SA3 + s * (SA4 + s * (SA5 + s * (SA6 + s * (SA7 + s * SA8)))))));
        rr / ss
    } else {
        let rr = RB0 + s * (RB1 + s * (RB2 + s * (RB3 + s * (RB4 + s * (RB5 + s * RB6)))));
        let ss =
            1.0 + s * (SB1 + s * (SB2 + s * (SB3 + s * (SB4 + s * (SB5 + s * (SB6 + s * SB7))))));
        rr / ss
    };

    let result = (-0.5625 + r).exp() / ax;

    if x < 0.0 {
        2.0 * (ax * ax).exp() - result
    } else {
        result
    }
}

/// Compute erfc(|x|) for |x| >= 1.25.
///
/// Uses the fdlibm technique:
///   erfc(x) = exp(-z^2 - 0.5625) * exp((z-x)*(z+x) + R/S) / x
/// where z is x with the lower 32 bits zeroed (for precision splitting).
fn erfc_inner(ax: f64) -> f64 {
    let s = 1.0 / (ax * ax);

    let r = if ax < 2.857142857142857 {
        let rr =
            RA0 + s * (RA1 + s * (RA2 + s * (RA3 + s * (RA4 + s * (RA5 + s * (RA6 + s * RA7))))));
        let ss = 1.0
            + s * (SA1
                + s * (SA2 + s * (SA3 + s * (SA4 + s * (SA5 + s * (SA6 + s * (SA7 + s * SA8)))))));
        rr / ss
    } else {
        let rr = RB0 + s * (RB1 + s * (RB2 + s * (RB3 + s * (RB4 + s * (RB5 + s * RB6)))));
        let ss =
            1.0 + s * (SB1 + s * (SB2 + s * (SB3 + s * (SB4 + s * (SB5 + s * (SB6 + s * SB7))))));
        rr / ss
    };

    // Zero lower 32 bits of ax for precision (fdlibm SET_LOW_WORD(z,0) trick)
    let z = f64::from_bits(ax.to_bits() & 0xFFFF_FFFF_0000_0000);
    let del = (ax - z) * (ax + z);
    (-z * z - 0.5625).exp() * (-del + r).exp() / ax
}
