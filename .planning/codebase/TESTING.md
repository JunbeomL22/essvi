# Testing Patterns

**Analysis Date:** 2026-03-05

## Test Framework

**Runner:**
- `cargo test` - Native Rust test framework built-in to Cargo
- Edition: 2024 (declared in `Cargo.toml`)

**Assertion Library:**
- Built-in `assert!()`, `assert_eq!()`, `assert!()` macros
- No external assertion library; uses Rust standard assertions

**Run Commands:**
```bash
cargo test                    # Run all unit and integration tests
cargo test --lib             # Run library tests only
cargo test --test '*'        # Run integration tests
cargo test --release         # Run tests in release mode
cargo bench                   # Run benchmarks via criterion
cargo test -- --nocapture    # Show println! output during tests
```

## Test File Organization

**Location:**
- Unit tests colocated with source code using `#[cfg(test)] mod tests { ... }`
- Integration tests in separate `tests/` directory at crate root
- Benchmark code in `benches/` directory

**Naming:**
- Unit test modules named `tests` within each source file
- Integration test file: `tests/steep_skew.rs`
- Benchmark file: `benches/calibration.rs`
- Individual test functions prefixed with module-like naming: `#[test]` attribute indicates test

**Structure:**
```
essvi/
├── src/
│   ├── lib.rs          # Module declarations only
│   ├── ssvi.rs         # SSVI functions + inline #[cfg(test)] mod tests
│   ├── calibration.rs  # Calibration logic + inline #[cfg(test)] mod tests
│   ├── brent.rs        # Root finder + inline #[cfg(test)] mod tests
│   ├── nelder_mead.rs  # Optimizer + inline #[cfg(test)] mod tests
│   └── bin/
│       └── report.rs   # Binary for report generation (not tested)
├── tests/
│   └── steep_skew.rs   # Integration test: stress test with steep skew scenarios
└── benches/
    └── calibration.rs  # Criterion benchmarks
```

## Test Structure

**Suite Organization:**
```rust
// Pattern from src/ssvi.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_basic() {
        // Arrange - setup inputs
        // Act - call function
        // Assert - verify result
    }

    #[test]
    fn atm_total_variance() {
        // Test another specific case
    }
}
```

**Patterns:**
- Setup via local variables and helper functions
- No explicit teardown needed (functions are pure)
- Helper functions defined within test module: `make_sample_slice()` in `calibration.rs` tests
- Tests organized by functional area, not alphabetically

## Mocking

**Framework:** None used

**Patterns:**
- No explicit mocking framework (mockito, etc.) used
- Pure functions make mocking unnecessary
- Test data generated synthetically:
```rust
// From calibration.rs tests
fn make_sample_slice() -> (Vec<f64>, Vec<f64>, f64, f64) {
    let true_eta = 0.8;
    let true_gamma = 0.4;
    let true_rho = -0.35;
    let theta_star = 0.04;
    let k_star = -0.01;

    let true_theta = solve_theta(true_eta, true_gamma, true_rho, theta_star, k_star)
        .expect("true theta must solve");

    let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
    let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

    (k_slice, w_market, theta_star, k_star)
}
```

**What to Mock:**
- Nothing in current codebase; all functions are pure mathematical functions

**What NOT to Mock:**
- All core computational functions should be tested directly
- Mathematical results are deterministic and verifiable

## Fixtures and Factories

**Test Data:**
```rust
// From benches/calibration.rs - benchmark helper
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

// From tests/steep_skew.rs - integration test scenario builder
fn make_steep_skew_slice(t_expiry: f64) -> (Vec<f64>, Vec<f64>, f64) {
    let k_slice: Vec<f64> = (0..20).map(|i| -0.7 + (i as f64) * 1.0 / 19.0).collect();

    let iv: Vec<f64> = k_slice
        .iter()
        .map(|&k| {
            if k <= 0.0 {
                0.4 + (0.7 - 0.4) * (-k / 0.7)
            } else {
                0.4 + 0.05 * (k / 0.3)
            }
        })
        .collect();

    let w_market: Vec<f64> = iv.iter().map(|&sigma| sigma * sigma * t_expiry).collect();
    let theta_star = 0.4 * 0.4 * t_expiry;

    (k_slice, w_market, theta_star)
}
```

**Location:**
- Test helpers defined in test module or at top of test file
- Parametric data generation via loop indices

## Coverage

**Requirements:** None enforced

**View Coverage:** Not configured

**Current Coverage:**
- Unit tests exist for core functions: `phi()`, `total_variance()`, `solve_theta()`, `calibrate()`, `brent()`, `nelder_mead_bounded()`
- Mathematical properties verified: ATM consistency, no-arbitrage constraints, parameter recovery
- Multiple test cases per function showing different scenarios

## Test Types

**Unit Tests:**
- Scope: Individual functions and small algorithmic units
- Approach: Direct function calls with known inputs, asserting expected outputs
- Locations: `src/ssvi.rs`, `src/calibration.rs`, `src/brent.rs`, `src/nelder_mead.rs`
- Example:
```rust
// From ssvi.rs
#[test]
fn phi_basic() {
    let p = phi(1.0, 1.0, 0.5);
    assert!((p - 1.0 / 2.0_f64.sqrt()).abs() < 1e-12);
}
```

**Integration Tests:**
- Scope: End-to-end calibration scenarios with realistic market data
- Approach: Synthetic market data generation, full calibration pipeline, result validation
- Location: `tests/steep_skew.rs`
- Example:
```rust
// From tests/steep_skew.rs
#[test]
fn calibrate_recovers_parameters() {
    let (k_slice, w_market, theta_star, k_star) = make_sample_slice();

    let res = calibrate(&input, &NelderMeadConfig::default())
        .expect("calibration must succeed");

    assert!(res.optimizer.converged, "optimizer did not converge");
    assert!(res.optimizer.f < 1e-20, "residual too large: {}", res.optimizer.f);
    assert!((res.rho - (-0.35)).abs() < 1e-3, "rho: {}", res.rho);
}
```

**E2E Tests:**
- Not formally defined; integration tests serve this purpose
- Binary report generator (`src/bin/report.rs`) tested manually via plotting output

## Common Patterns

**Async Testing:**
- Not applicable (no async code in codebase)

**Error Testing:**
```rust
// From brent.rs tests - testing failure case
#[test]
fn no_sign_change() {
    let res = brent(|x| x * x + 1.0, 0.0, 2.0, 1e-14, 100);
    assert!(!res.converged);
}

// From calibration.rs tests - testing edge case
#[test]
fn solve_theta_basic() {
    let theta = solve_theta(0.8, 0.4, -0.35, 0.04, 0.0);
    assert!(theta.is_some());
    let t = theta.unwrap();
    assert!((t - 0.04).abs() < 1e-10);
}

// Option-based error handling in tests
#[test]
fn calibrate_recovers_parameters() {
    // ... setup ...
    let res = calibrate(&input, &NelderMeadConfig::default())
        .expect("calibration must succeed");  // Panics if None
}
```

**Floating-Point Assertions:**
```rust
// Pattern: using absolute tolerance for float comparison
assert!((p - 1.0 / 2.0_f64.sqrt()).abs() < 1e-12);
assert!((res.rho - (-0.35)).abs() < 1e-3, "rho: {}", res.rho);
assert!((recovered_phi - true_phi).abs() / true_phi < 1e-3);

// Pattern: relative tolerance for large values
assert!(max_err < 1e-10, "max pointwise error: {}", max_err);
```

**Property-Based Testing:**
```rust
// From calibration.rs - synthetic data verification pattern
// Generate from known parameters → calibrate → verify recovery
let true_eta = 0.8;
let true_gamma = 0.4;
let true_rho = -0.35;

let true_theta = solve_theta(true_eta, true_gamma, true_rho, theta_star, k_star)
    .expect("true theta must solve");

let k_slice: Vec<f64> = (0..20).map(|i| -0.5 + (i as f64) / 19.0).collect();
let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

// ... calibrate back and verify recovery ...
assert!((res.rho - (-0.35)).abs() < 1e-3, "rho: {}", res.rho);
let recovered_phi = ssvi::phi(res.theta, res.eta, res.gamma);
assert!((recovered_phi - true_phi).abs() / true_phi < 1e-3);
```

## Benchmarks

**Framework:** Criterion.rs

**Location:** `benches/calibration.rs`

**Configuration:** In `Cargo.toml`:
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "calibration"
harness = false
```

**Benchmark Functions:**
```rust
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
```

**Run Benchmarks:**
```bash
cargo bench                    # Run all benchmarks
cargo bench -- --verbose       # Show detailed output
cargo bench -- --baseline foo  # Compare against saved baseline
```

**Report Output:** HTML reports generated in `target/criterion/` directory

---

*Testing analysis: 2026-03-05*
