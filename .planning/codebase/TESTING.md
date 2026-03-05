# Testing Patterns

**Analysis Date:** 2026-03-05

## Test Framework

**Runner:**
- Rust built-in test framework (`#[test]` attribute)
- No external test dependencies in `Cargo.toml`

**Assertion Library:**
- Standard library macros: `assert_eq!`, `assert!`, `assert_ne!`
- For floating-point comparisons, use approximate equality (see patterns below)

**Run Commands:**
```bash
cargo test                 # Run all tests
cargo test -- --nocapture  # Run with stdout visible
cargo test <test_name>     # Run specific test
```

## Test File Organization

**Location:**
- Co-located within source files using `#[cfg(test)] mod tests` blocks
- Tests live at the bottom of the file they test

**Naming:**
- Test module: `mod tests` (standard Rust convention)
- Test functions: snake_case descriptive names prefixed with context

**Structure:**
```
src/
├── lib.rs          # Contains `mod tests` at bottom
├── [module].rs     # Each module contains its own `mod tests`
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
```

**Patterns:**
- Import parent module with `use super::*`
- Each `#[test]` function tests one behavior
- No setup/teardown fixtures (not needed for pure numerical functions)
- Arrange-Act-Assert pattern

## Recommended Test Patterns for This Codebase

**Floating-Point Approximate Equality:**
```rust
#[test]
fn test_optimization_converges() {
    let result = nelder_mead_bounded(/* ... */);
    assert!(result.converged);
    assert!((result.f - expected).abs() < 1e-10,
        "Expected f ~ {}, got {}", expected, result.f);
}
```

**Testing Convergence Properties:**
```rust
#[test]
fn test_rosenbrock_2d() {
    let result = nelder_mead_bounded(
        |x| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0].powi(2)).powi(2),
        &[0.0, 0.0],
        &[-5.0, -5.0],
        &[5.0, 5.0],
        &NelderMeadConfig::default(),
    );
    assert!(result.converged);
    assert!((result.x[0] - 1.0).abs() < 1e-6);
    assert!((result.x[1] - 1.0).abs() < 1e-6);
}
```

**Testing Boundary Solutions:**
```rust
#[test]
fn test_solution_at_boundary() {
    let result = nelder_mead_bounded(
        |x| x[0] + x[1],      // minimum at lower bounds
        &[1.0, 1.0],
        &[0.0, 0.0],
        &[2.0, 2.0],
        &NelderMeadConfig::default(),
    );
    assert!(result.converged);
    assert!((result.x[0] - 0.0).abs() < 1e-6);
    assert!((result.x[1] - 0.0).abs() < 1e-6);
}
```

**Testing SSVI-Specific Functions:**
```rust
#[test]
fn test_ssvi_no_arbitrage() {
    // Verify eta * (1 + |rho|) <= 2
    let eta = result.x[0];
    let rho = result.x[2];
    assert!(eta * (1.0 + rho.abs()) <= 2.0 + 1e-10);
}
```

## Mocking

**Framework:** Not applicable

**Guidance:**
- This is a pure numerical computation library with no external dependencies or I/O
- No mocking is needed; test functions directly with known inputs and expected outputs
- Use well-known test functions (Rosenbrock, Sphere, Booth) as validation benchmarks

## Fixtures and Factories

**Test Data:**
```rust
// Use inline constants for mathematical test cases
const THETA_STAR: f64 = 0.04;
const K_STAR: f64 = 0.0;

fn default_test_config() -> NelderMeadConfig {
    NelderMeadConfig {
        max_iter: 1000,
        tol_f: 1e-12,
        tol_x: 1e-12,
        ..Default::default()
    }
}
```

**Location:**
- Define test helpers and constants inside `mod tests` blocks
- For shared test utilities across modules, create `src/test_utils.rs` (gated with `#[cfg(test)]`)

## Coverage

**Requirements:** None enforced currently

**View Coverage:**
```bash
cargo install cargo-tarpaulin    # One-time install
cargo tarpaulin                  # Generate coverage report
cargo tarpaulin --out Html       # HTML coverage report
```

## Test Types

**Unit Tests:**
- Co-located `#[cfg(test)] mod tests` blocks
- Test individual functions with known mathematical inputs/outputs
- Verify convergence, accuracy, and boundary behavior

**Integration Tests:**
- Place in `tests/` directory at project root (not yet created)
- Use for end-to-end SSVI calibration validation against known market data
- Test the full pipeline: input market data -> calibrate -> verify parameters

**E2E Tests:**
- Not applicable (library crate, not a binary)

**Benchmark Tests:**
- Consider `criterion` crate for performance benchmarking optimization routines
- Place in `benches/` directory

## Common Patterns

**Numerical Accuracy Testing:**
```rust
#[test]
fn test_with_tolerance() {
    let result = compute_something();
    let expected = 3.14159;
    let tol = 1e-10;
    assert!(
        (result - expected).abs() < tol,
        "Expected {:.15} but got {:.15}, diff = {:.2e}",
        expected, result, (result - expected).abs()
    );
}
```

**Convergence Testing:**
```rust
#[test]
fn test_convergence_within_iterations() {
    let result = nelder_mead_bounded(/* ... */);
    assert!(result.converged, "Failed to converge after {} iterations", result.iterations);
    assert!(result.iterations < 500, "Too many iterations: {}", result.iterations);
}
```

**Edge Case Testing:**
```rust
#[test]
fn test_already_at_minimum() {
    let result = nelder_mead_bounded(
        |x| x[0].powi(2),
        &[0.0],           // start at minimum
        &[-1.0],
        &[1.0],
        &NelderMeadConfig::default(),
    );
    assert!(result.f < 1e-12);
}
```

## Key Test Cases from Guideline

The `documents/guideline.md` specifies these validation benchmarks:

| Test Case | Expected Iterations | Expected Accuracy |
|-----------|:------------------:|:-----------------:|
| Rosenbrock 2D | ~125 | ~1e-13 |
| Bounded (solution at boundary) | ~7 | ~1e-6 |
| SSVI-like 3D | ~193 | ~1e-13 |

Implement these as regression tests to verify optimizer correctness.

---

*Testing analysis: 2026-03-05*
