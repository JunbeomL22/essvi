# Testing Patterns

**Analysis Date:** 2026-03-08

## Test Framework

**Runner:**
- Rust built-in test framework (`#[test]` attribute)
- No external test runner crate

**Benchmark Framework:**
- Criterion 0.5 with `html_reports` feature
- Config: `Cargo.toml` under `[dev-dependencies]`
- Benchmark harness disabled: `harness = false` in `[[bench]]`

**Assertion Library:**
- Standard `assert!`, `assert_eq!`, `assert!(matches!(...))` macros
- No external assertion crate

**Run Commands:**
```bash
cargo test                    # Run all tests (unit + integration + doctests)
cargo test -- --nocapture     # Run with stdout visible (for stress tests with println!)
cargo test test_name          # Run specific test
cargo bench                   # Run criterion benchmarks
cargo bench -- benchmark_name # Run specific benchmark
```

## Test File Organization

**Location:**
- Integration tests in `tests/` directory (separate from source)
- No co-located unit tests inside `src/` modules (all tests are external integration tests)
- Doc tests embedded in source file doc comments (`///` blocks)
- Benchmarks in `benches/` directory

**Naming:**
- Test files named after the module they test: `tests/ssvi.rs`, `tests/calibration.rs`, `tests/pricing.rs`
- Cross-cutting test files named after the concern: `tests/steep_skew.rs`, `tests/implied_vol.rs`
- Benchmark file named after the subsystem: `benches/calibration.rs`

**Structure:**
```
tests/
├── brent.rs              # 2 tests: Brent root finder
├── calibration.rs        # 5 tests: SSVI calibration pipeline
├── crude_solver.rs       # 8 tests: 4D crude solver + surface calibration
├── implied_vol.rs        # 18 tests: Let's Be Rational IV solver
├── math.rs               # 28 tests: erf, erfc, normal CDF/PDF/inverse
├── nelder_mead.rs        # 2 tests: Nelder-Mead optimizer
├── pricing.rs            # 31 tests: Black-76 pricing + greeks + errors
├── ssvi.rs               # 3 tests: SSVI model functions
└── steep_skew.rs         # 2 tests: stress tests for steep skew

benches/
└── calibration.rs        # 4 benchmark groups
```

## Test Counts

| Category | Count |
|----------|------:|
| Integration tests | 99 |
| Doc tests | 20 |
| **Total** | **119** |

## Test Structure

**Suite Organization:**
```rust
/// Module-level doc comment describing what this file tests.
use essvi::calibration::{CalibrationConfig, CalibrationInput, calibrate, solve_theta};
use essvi::model::ssvi;

/// Helper: generate synthetic data for testing.
fn make_sample_slice() -> (Vec<f64>, Vec<f64>, f64, f64) {
    // Build test data from known parameters
    let true_eta = 0.8;
    // ...
    (k_slice, w_market, theta_star, k_star)
}

#[test]
fn descriptive_test_name() {
    let (k_slice, w_market, theta_star, k_star) = make_sample_slice();
    // ... test body ...
    assert!((result - expected).abs() < tolerance,
        "helpful message: actual={}, expected={}", result, expected);
}
```

**Patterns:**
- Each test file begins with a module-level `///` doc comment
- Imports use the public API via `essvi::` crate path (testing as external consumer)
- Shared test data builders defined as plain `fn` helpers at the top of each file
- No setup/teardown traits or fixtures framework
- No `#[cfg(test)]` blocks in library source (all tests are external)

## Test Naming Conventions

**Format:** `snake_case` describing what is being tested:
- `solve_theta_basic` -- basic correctness
- `solve_theta_nonzero_kstar` -- specific input condition
- `calibrate_recovers_parameters` -- behavior verification
- `no_arbitrage_enforced` -- constraint verification
- `implied_vol_atm_round_trip` -- round-trip property
- `normalised_black_call_put_call_parity` -- mathematical identity

**Prefixes for crude_solver tests:** `test_` prefix used consistently:
- `test_crude_calib_config_default`
- `test_calibrate_slice_synthetic_flat`
- `test_calibrate_surface_monotonic_theta`

**Note:** The `test_` prefix is used in `tests/crude_solver.rs` but NOT in other test files. For new tests, follow the majority pattern and omit the `test_` prefix.

## Assertion Patterns

**Numerical tolerance assertions:**
```rust
// Exact mathematical identity: tight tolerance
assert!((w - 0.04).abs() < 1e-12, "w={}, expected 0.04", w);

// Round-trip recovery: moderate tolerance
assert!((iv - sigma).abs() < 1e-10, "round-trip: sigma={}, recovered={}", sigma, iv);

// Optimization recovery: looser tolerance
assert!((res.rho - (-0.35)).abs() < 1e-3, "rho: {}", res.rho);

// Phi equivalence (eta/gamma degeneracy): relative tolerance
assert!((recovered_phi - true_phi).abs() / true_phi < 1e-3,
    "phi: {} vs true {}", recovered_phi, true_phi);
```

**Tolerance guide for new tests:**
| Scenario | Tolerance |
|----------|-----------|
| Exact math (CDF, erf) | `1e-14` to `1e-15` |
| Round-trip (price -> IV -> price) | `1e-10` to `1e-12` |
| Optimizer convergence (residual) | `1e-18` to `1e-20` |
| Parameter recovery (eta, rho) | `1e-3` |
| Phi recovery (relative) | `1e-3` relative |
| Crude solver parameter recovery | `0.01` to `0.15` |
| Total variance pointwise | `1e-6` to `1e-10` |

**Error type assertions:**
```rust
assert!(matches!(result, Err(PricingError::BelowIntrinsic { .. })),
    "Expected BelowIntrinsic error, got {:?}", result);

assert!(matches!(r1, Err(PricingError::InvalidInput(_))));
```

**Boolean property assertions:**
```rust
assert!(res.optimizer.converged, "optimizer did not converge");
assert!(ssvi::no_arbitrage_satisfied(res.eta, res.rho),
    "no-arb violated: eta={}, rho={}", res.eta, res.rho);
assert!(g >= -1e-6, "butterfly violation at k={:.4}: g={:.6e}", k, g);
```

**Always include diagnostic messages** in assertions with the actual vs expected values. This is a strong convention in the codebase.

## Test Data Generation

**Synthetic data from known parameters:**
```rust
fn make_sample_slice() -> (Vec<f64>, Vec<f64>, f64, f64) {
    let true_eta = 0.8;
    let true_gamma = 0.4;
    let true_rho = -0.35;
    let theta_star = 0.04;
    let k_star = -0.01;

    let config = CalibrationConfig::default();
    let true_theta = solve_theta(true_eta, true_gamma, true_rho, theta_star, k_star, &config)
        .expect("true theta must solve");

    let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
    let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

    (k_slice, w_market, theta_star, k_star)
}
```

**Pattern:** Generate data from known SSVI parameters, then verify the calibrator recovers them. This ensures tests are noise-free and have a known ground truth.

**Reference values from external sources:**
```rust
// Reference: scipy.stats Black-76 ATM call
assert!((c - 7.965567455405804).abs() < 1e-10);

// (x, erf(x)) pairs from Abramowitz & Stegun Table 7.1
let cases: &[(f64, f64)] = &[
    (0.25, 2.76326390168236901e-01),
    // ...
];
```

**Location:**
- Test helpers defined directly in each test file (no shared test utilities crate)
- No `tests/common/` module or shared fixture files

## Mocking

**Framework:** None. No mocking is used.

**Why:** The codebase is purely numerical/mathematical. All functions are pure (no I/O, no side effects in library code). Tests use synthetic data generated from known parameters instead of mocking.

**External dependencies in tests:** None. The only external dependency (`plotters`) is not tested directly. All tests exercise the pure mathematical library.

## Coverage

**Requirements:** None enforced. No coverage thresholds configured.

**View Coverage:**
```bash
# Requires cargo-tarpaulin or cargo-llvm-cov
cargo tarpaulin --out Html     # HTML coverage report
cargo llvm-cov --html          # Alternative with llvm-cov
```

**Current coverage by module (qualitative assessment):**

| Module | Test Coverage | Key Gaps |
|--------|-------------|----------|
| `model/ssvi` | High -- `phi`, `total_variance`, `no_arbitrage_satisfied` all tested | `total_variance_slice` tested indirectly |
| `calibration` | High -- `solve_theta`, `calibrate`, `calibrate_with_calendar_penalty` tested | Weighted error path tested via benchmarks only |
| `crude_solver` | High -- `calibrate_slice`, `calibrate_surface`, config defaults, butterfly, calendar penalty | `butterfly_density` tested indirectly |
| `solver/nelder_mead` | Medium -- Rosenbrock + boundary test | Limited edge cases (non-convergence, high dimensions) |
| `solver/brent` | Medium -- sqrt(2) + no-sign-change | Limited edge cases |
| `pricing/black76` | Very high -- price, delta, gamma, vega, theta, greeks, discounted_price, all error paths | Complete |
| `pricing/lets_be_rational` | Very high -- round-trips across strikes/vols/expiries, error paths | Deep OTM edge cases |
| `pricing/rational_cubic` | None -- no direct tests | Exercised indirectly through IV solver |
| `math/erf` | High -- erf, erfc, erfcx with reference values, symmetry, boundary | Complete |
| `math/normal` | High -- PDF, CDF, inverse CDF with reference values | Complete |
| `math/normal_hp` | High -- CDF tail behavior, round-trip inverse | Complete |
| `math/constants` | Medium -- epsilon, sqrt(2pi) verified | Not all constants checked |
| `fit_common` | None -- no tests for `make_slice`, `build_market_slices`, `plot_fit` | Binary helper, not critical |

## Test Types

**Unit Tests (Integration-style, in `tests/`):**
- All 99 tests test individual functions via the public API
- Scope: single function or small function chain
- Examples: `tests/ssvi.rs`, `tests/math.rs`, `tests/pricing.rs`

**Property Tests:**
- Symmetry: `erf(-x) = -erf(x)`, `norm_pdf(x) = norm_pdf(-x)`
- Monotonicity: `norm_cdf` is monotonically increasing
- Complement: `erf(x) + erfc(x) = 1`
- Put-call parity: `C - P = F - K`
- Round-trip: `implied_vol(price(sigma)) = sigma`
- Boundary: `delta_call in [0,1]`, `delta_put in [-1,0]`
- Consistency: `greeks()` matches individual greek functions
- Numerical derivative: `gamma` matches `d(delta)/dF`, `vega` matches `dP/dsigma`

**Stress Tests:**
- `tests/steep_skew.rs`: Tests calibration under extreme conditions (very short expiry, steep skew)
- Parametric: iterates over multiple expiries `[1.0, 0.1, 0.01, 0.001]`
- Outputs diagnostic information via `println!` (visible with `--nocapture`)

**Regression Tests:**
- `tests/crude_solver.rs::test_calibrate_surface_preserves_existing_behavior`: Verifies refactoring did not change behavior

**E2E Tests:**
- Not used. Binary programs are not tested via the test framework.

## Benchmark Patterns

**Framework:** Criterion 0.5

**Config:** `benches/calibration.rs`

**Benchmark structure:**
```rust
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_calibrate_20pt(c: &mut Criterion) {
    let (k_slice, w_market, theta_star, k_star) = make_20pt_slice();
    let input = CalibrationInput { /* ... */ };
    let config = CalibrationConfig::default();

    c.bench_function("calibrate_20pt_slice", |b| {
        b.iter(|| calibrate(black_box(&input), black_box(&config)))
    });
}

criterion_group!(benches, bench_calibrate_20pt, bench_solve_theta, ...);
criterion_main!(benches);
```

**Benchmarks defined:**
| Benchmark | What it measures |
|-----------|-----------------|
| `calibrate_20pt_slice` | Single-slice SSVI calibration (20 points) |
| `solve_theta/k_star=X` | Newton solver for theta at various k_star values |
| `total_variance_20pt` | SSVI model evaluation (20 points) |
| `surface_12_slices` | Full surface calibration (12 slices, 60 points each) |

**Pattern:** Data is prepared outside the benchmark closure; only the hot path is measured. `black_box()` prevents compiler optimization of inputs.

## Common Test Patterns

**Parametric testing (manual):**
```rust
#[test]
fn implied_vol_various_strikes() {
    let f = 100.0;
    let t = 1.0;
    let sigma = 0.20;
    for &k in &[80.0, 90.0, 95.0, 100.0, 105.0, 110.0, 120.0] {
        for &q in &[1, -1] {
            let price = black76::price(f, k, sigma, t, q).unwrap();
            if price < 1e-15 { continue; }
            let iv = lets_be_rational::implied_volatility(price, f, k, t, q).unwrap();
            assert!((iv - sigma).abs() < 1e-8,
                "K={}, q={}: sigma={}, recovered={}", k, q, sigma, iv);
        }
    }
}
```

**Error path testing:**
```rust
#[test]
fn error_negative_forward() {
    let result = black76::price(-100.0, 100.0, 0.20, 1.0, 1);
    assert!(matches!(result, Err(PricingError::InvalidInput(_))));
}
```

**Degenerate case testing:**
```rust
#[test]
fn zero_vol_call() {
    let c = black76::price(100.0, 90.0, 0.0, 1.0, 1).unwrap();
    assert!((c - 10.0).abs() < 1e-15, "Zero-vol ITM call should be 10, got {}", c);
}
```

**Numerical derivative verification:**
```rust
#[test]
fn gamma_numerical_check() {
    let eps = 1e-6;
    let d_up = black76::delta(f + eps, k, sigma, t, 1).unwrap();
    let d_dn = black76::delta(f - eps, k, sigma, t, 1).unwrap();
    let gamma_num = (d_up - d_dn) / (2.0 * eps);
    let gamma_exact = black76::gamma(f, k, sigma, t).unwrap();
    assert!((gamma_exact - gamma_num).abs() < 1e-6);
}
```

## Writing New Tests

**Where to add:**
- Tests for a new module `src/foo.rs`: create `tests/foo.rs`
- Tests for existing module `src/calibration.rs`: add to `tests/calibration.rs`
- Stress/edge-case tests: create dedicated file like `tests/steep_skew.rs`

**Checklist for new numerical tests:**
1. Generate synthetic data from known parameters
2. Assert recovery to appropriate tolerance (see tolerance guide above)
3. Include diagnostic info in assertion messages
4. Test degenerate/edge cases (zero vol, zero time, boundary values)
5. Test error paths with `assert!(matches!(result, Err(...)))`
6. For greeks: verify against numerical finite differences

---

*Testing analysis: 2026-03-08*
