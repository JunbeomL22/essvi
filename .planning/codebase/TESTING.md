# Testing Patterns

**Analysis Date:** 2026-03-07

## Test Framework

**Runner:**
- Built-in Rust test framework (`#[cfg(test)]` + `#[test]`)
- No additional test harness dependencies for unit/integration tests
- Config: None (default `cargo test` behavior)

**Assertion Library:**
- Standard `assert!`, `assert_eq!`, `assert_ne!` macros
- Custom floating-point tolerance checks via `assert!((value - expected).abs() < tol)`

**Benchmark Framework:**
- Criterion 0.5 with `html_reports` feature
- Config: `Cargo.toml` `[[bench]]` section, `harness = false`

**Run Commands:**
```bash
cargo test                    # Run all unit + integration tests
cargo test -- --nocapture     # Run with stdout visible (for diagnostic prints)
cargo test steep_skew         # Run specific test module/function
cargo bench                   # Run all benchmarks (Criterion)
cargo bench -- calibrate      # Run specific benchmark
```

## Test File Organization

**Location:**
- Unit tests: co-located in source files via `#[cfg(test)] mod tests { ... }`
- Integration tests: `tests/` directory (standard Rust convention)
- Benchmarks: `benches/` directory

**Naming:**
- Unit test modules: always `mod tests` inside `#[cfg(test)]`
- Integration test files: descriptive scenario name (`steep_skew.rs`)
- Benchmark files: match the module under test (`calibration.rs`)

**Structure:**
```
src/
  ssvi.rs           # Contains #[cfg(test)] mod tests (3 unit tests)
  calibration.rs    # Contains #[cfg(test)] mod tests (5 unit tests)
  nelder_mead.rs    # Contains #[cfg(test)] mod tests (2 unit tests)
  brent.rs          # Contains #[cfg(test)] mod tests (2 unit tests)
tests/
  steep_skew.rs     # Integration test (2 tests): stress-test calibration
benches/
  calibration.rs    # Criterion benchmarks (4 bench functions)
```

## Test Structure

**Unit Test Organization:**
```rust
// From src/calibration.rs
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: generate synthetic data from known parameters.
    fn make_sample_slice() -> (Vec<f64>, Vec<f64>, f64, f64) {
        // ... build test fixtures inline
    }

    #[test]
    fn solve_theta_basic() {
        // Test degenerate/simple case first
        let theta = solve_theta(0.8, 0.4, -0.35, 0.04, 0.0);
        assert!(theta.is_some());
        let t = theta.unwrap();
        assert!((t - 0.04).abs() < 1e-10);
    }

    #[test]
    fn calibrate_recovers_parameters() {
        // Round-trip test: generate from known params, calibrate back
        let (k_slice, w_market, theta_star, k_star) = make_sample_slice();
        // ... calibrate and assert recovery
    }
}
```

**Patterns:**
- Each test function verifies one property or scenario
- Helper functions build test data inline (no external fixtures)
- Tests progress from simple/degenerate cases to complex scenarios
- Round-trip testing: generate synthetic data from known parameters, calibrate, verify recovery

**Assertion Patterns for Floating-Point:**
```rust
// Absolute tolerance check
assert!((value - expected).abs() < 1e-12);

// Relative tolerance check
assert!(
    (recovered_phi - true_phi).abs() / true_phi < 1e-3,
    "phi: {} vs true {}", recovered_phi, true_phi
);

// Residual magnitude check
assert!(res.optimizer.f < 1e-20, "residual too large: {}", res.optimizer.f);

// Boolean property check
assert!(res.optimizer.converged, "optimizer did not converge");
assert!(ssvi::no_arbitrage_satisfied(res.eta, res.rho));
```

**Custom assertion messages:** Always include diagnostic values in assertion messages using `format!` arguments:
```rust
assert!((w - 0.04).abs() < 1e-12, "w={}, theta*=0.04, diff={}", w, (w - 0.04).abs());
assert!(res.optimizer.f < 1e-18, "residual: {}", res.optimizer.f);
```

## Integration Tests

**Location:** `tests/steep_skew.rs`

**Pattern:** Stress-test the full calibration pipeline under extreme conditions.

```rust
// From tests/steep_skew.rs
use essvi::calibration::{calibrate, CalibrationInput};
use essvi::nelder_mead::NelderMeadConfig;
use essvi::ssvi;

fn make_steep_skew_slice(t_expiry: f64) -> (Vec<f64>, Vec<f64>, f64) {
    // Build synthetic market data with extreme parameters
}

#[test]
fn steep_skew_stress_test() {
    // Test across multiple expiries from 1Y down to near-zero
    let expiries = [1.0, 0.1, 0.01, 0.001];
    for &t in &expiries {
        run_calibration("Steep skew", t);
    }
}

#[test]
fn steep_skew_fit_quality() {
    // Quantitative assertions for each regime
    let expiries = [1.0, 0.1, 0.01, 0.001];
    for &t in &expiries {
        // ... calibrate and check fit quality
        assert!(res.is_some(), "calibration failed at T={}", t);
    }
}
```

**Integration test characteristics:**
- Import the crate as an external consumer: `use essvi::calibration::...`
- Test across parameter sweeps (multiple expiries, slopes)
- Include diagnostic `println!` output (visible with `--nocapture`)
- One test for visual/diagnostic output, one for quantitative assertions

## Mocking

**Framework:** None

**What to Mock:** Nothing -- the codebase is pure numerical computation with no external dependencies, I/O, or side effects in library code.

**Testing strategy instead of mocking:**
- Generate synthetic data from known parameters (round-trip testing)
- Use well-known mathematical functions as test cases (Rosenbrock for optimizer, sqrt(2) for root finder)
- Verify mathematical properties (no-arbitrage condition, ATM consistency)

## Fixtures and Factories

**Test Data:**
- All test data generated inline via helper functions
- No external fixture files, no JSON/CSV test data

**Factory pattern used consistently:**
```rust
// Unit test factories (private to test module)
fn make_sample_slice() -> (Vec<f64>, Vec<f64>, f64, f64) { ... }

// Integration test factories
fn make_steep_skew_slice(t_expiry: f64) -> (Vec<f64>, Vec<f64>, f64) { ... }

// Benchmark factories
fn make_20pt_slice() -> (Vec<f64>, Vec<f64>, f64, f64) { ... }
fn make_surface_slices() -> Vec<(Vec<f64>, Vec<f64>, f64, f64, f64)> { ... }
```

**Return type convention:** Factories return tuples of `(k_slice, w_market, theta_star, k_star)` -- always in this order.

## Benchmarks

**Framework:** Criterion 0.5 (`benches/calibration.rs`)

**Pattern:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_calibrate_20pt(c: &mut Criterion) {
    // Setup outside the benchmark loop
    let (k_slice, w_market, theta_star, k_star) = make_20pt_slice();
    let input = CalibrationInput { ... };
    let config = NelderMeadConfig::default();

    c.bench_function("calibrate_20pt_slice", |b| {
        b.iter(|| calibrate(black_box(&input), black_box(&config)))
    });
}

criterion_group!(benches, bench_calibrate_20pt, bench_solve_theta, ...);
criterion_main!(benches);
```

**Benchmarked operations (4 benchmarks):**
- `calibrate_20pt_slice` -- full calibration of a single 20-point slice
- `solve_theta` -- Newton solver across 6 different k_star values (grouped)
- `total_variance_20pt` -- vectorized total variance computation
- `surface_12_slices` -- full 12-slice surface calibration with calendar penalty

**Anti-optimization:** All benchmark inputs wrapped in `black_box()` to prevent compiler elision.

## Coverage

**Requirements:** None enforced (no coverage tool configured)

**Current coverage assessment:**
- `src/ssvi.rs`: 3 unit tests covering `phi`, `total_variance` (ATM case), `no_arbitrage_satisfied`
- `src/calibration.rs`: 5 unit tests covering `solve_theta` (2 cases), `calibrate` (2 cases), no-arb enforcement
- `src/nelder_mead.rs`: 2 unit tests covering Rosenbrock convergence and boundary solutions
- `src/brent.rs`: 2 unit tests covering convergence (sqrt 2) and non-convergence (no sign change)
- `tests/steep_skew.rs`: 2 integration tests for stress scenarios
- `calibrate_with_calendar_penalty`: tested only in benchmarks, no unit or integration test assertions
- Binary code (`src/bin/`): no tests

**View Coverage:**
```bash
cargo install cargo-tarpaulin    # If not installed
cargo tarpaulin --out Html       # Generate HTML report
```

## Test Types

**Unit Tests (12 total):**
- Scope: individual mathematical functions and solver convergence
- Pattern: call function with known inputs, assert output within tolerance
- Location: `#[cfg(test)] mod tests` in each `src/*.rs` file

**Integration Tests (2 total):**
- Scope: full calibration pipeline under stress conditions
- Pattern: build realistic market data, run calibration, assert fit quality
- Location: `tests/steep_skew.rs`

**E2E Tests:** Not used. Binary outputs (reports, plots) are not tested programmatically.

**Benchmarks (4 total):**
- Scope: performance regression detection for hot paths
- Pattern: Criterion microbenchmarks with `black_box` inputs
- Location: `benches/calibration.rs`

## Common Patterns

**Round-Trip Testing:**
```rust
// Generate synthetic data from known true parameters
let true_theta = solve_theta(true_eta, true_gamma, true_rho, theta_star, k_star)
    .expect("true theta must solve");
let w_market = ssvi::total_variance_slice(&k_slice, true_theta, true_eta, true_gamma, true_rho);

// Calibrate back
let res = calibrate(&input, &NelderMeadConfig::default())
    .expect("calibration must succeed");

// Verify recovery (use phi since eta/gamma have degeneracy)
let recovered_phi = ssvi::phi(res.theta, res.eta, res.gamma);
assert!((recovered_phi - true_phi).abs() / true_phi < 1e-3);
```

**Degenerate Case Testing:**
```rust
// k_star = 0 simplifies the equation to theta = theta_star
let theta = solve_theta(0.8, 0.4, -0.35, 0.04, 0.0);
assert!((t - 0.04).abs() < 1e-10);

// ATM (k=0) total variance equals theta regardless of other params
let w = total_variance(0.0, 0.04, 1.0, 0.5, -0.3);
assert!((w - 0.04).abs() < 1e-12);
```

**Property-Based Assertions:**
```rust
// No-arbitrage must hold on calibrated output
assert!(ssvi::no_arbitrage_satisfied(res.eta, res.rho));

// Optimizer must converge
assert!(res.optimizer.converged);

// Pointwise model-vs-market error must be small
let max_err: f64 = w_fit.iter().zip(w_market.iter())
    .map(|(m, w)| (m - w).abs())
    .fold(0.0_f64, f64::max);
assert!(max_err < 1e-10);
```

**Parameterized Testing (manual loop):**
```rust
// No parameterized test macro used; manual loops over test cases
let expiries = [1.0, 0.1, 0.01, 0.001];
for &t in &expiries {
    let res = calibrate(&input, &NelderMeadConfig::default());
    assert!(res.is_some(), "calibration failed at T={}", t);
}
```

---

*Testing analysis: 2026-03-07*
