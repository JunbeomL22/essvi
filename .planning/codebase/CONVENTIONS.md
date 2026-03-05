# Coding Conventions

**Analysis Date:** 2026-03-05

## Naming Patterns

**Files:**
- Snake_case for Rust source files: `ssvi.rs`, `nelder_mead.rs`, `brent.rs`, `calibration.rs`
- Binary files in separate directory: `src/bin/report.rs`, `src/bin/check_eta.rs`
- Test integration files follow module pattern: `tests/steep_skew.rs`

**Functions:**
- Snake_case for all function names: `phi()`, `total_variance()`, `solve_theta()`, `calibrate()`, `nelder_mead_bounded()`, `brent()`
- Helper functions (private) also snake_case with leading underscore if non-public: `squared_error()` (private in `calibration.rs`)
- Factory/builder functions prefixed with `make_`: `make_market_data()`, `make_sample_slice()`, `make_20pt_slice()`
- Test helper functions follow same convention: `run_scenario()`, `run_calibration()`

**Variables:**
- Snake_case for all variables: `theta_star`, `w_market`, `k_slice`, `max_iv_err`, `rmse_iv`
- Single letters acceptable for mathematical variables: `k` (log-moneyness), `w` (total variance), `f` (function value), `x` (optimization variable)
- Boolean flags use descriptive names: `converged`, `mflag` (mathematical algorithm state)
- Loop counters: `i`, `j` for nested loops

**Types:**
- PascalCase for struct names: `NelderMeadConfig`, `NelderMeadResult`, `CalibrationInput`, `CalibrationResult`, `BrentResult`, `FitResult`, `Scenario`
- All public structs derive `Debug` and `Clone`: `#[derive(Debug, Clone)]`
- Field names follow snake_case: `k_slice`, `w_market`, `theta_star`, `max_iter`, `tol_f`, `converged`

**Constants & Configuration:**
- Numerical constants inline in functions, often using scientific notation: `1e-14`, `1e-15`, `1e-10`, `1e-3`
- Configuration defaults in `impl Default` blocks: `NelderMeadConfig`, `NelderMeadResult`
- Grid parameters hardcoded in functions: `n_rho = 20` for rho grid sweep in `calibrate()`

## Code Style

**Formatting:**
- Standard Rust formatting conventions (inferred from code structure)
- 4-space indentation
- No explicit rustfmt.toml file present
- Lines typically under 100 characters

**Linting:**
- No clippy configuration detected
- Code assumes default Rust 2024 edition as declared in `Cargo.toml`

## Import Organization

**Order:**
1. Standard library imports: `use std::fs;`, `use std::io::Write;`
2. External crate imports: `use criterion::*;`, `use plotters::prelude::*;`
3. Internal crate imports: `use crate::ssvi;`, `use crate::calibration::*;`, `use essvi::calibration::*;`
4. Self imports for tests: `use super::*;`

**Path Aliases:**
- No path alias configuration detected
- Relative imports used: `crate::ssvi`, `crate::brent`, `super::*`
- External package imports use full module paths: `essvi::calibration::CalibrationInput`, `essvi::ssvi`

**Examples from codebase:**
```rust
// src/calibration.rs
use crate::brent::brent;
use crate::nelder_mead::{nelder_mead_bounded, NelderMeadConfig, NelderMeadResult};
use crate::ssvi;

// src/bin/report.rs
use essvi::calibration::{calibrate, CalibrationInput};
use essvi::nelder_mead::NelderMeadConfig;
use essvi::ssvi;
use plotters::prelude::*;
use std::fs;
use std::io::Write;

// In test modules
use super::*;
```

## Error Handling

**Patterns:**
- `Option<T>` used extensively for fallible operations: `solve_theta()` returns `Option<f64>`, `calibrate()` returns `Option<CalibrationResult>`
- No explicit error types defined; uses idiomatic `Option` and `Result` patterns
- Unwrap in test/benchmark code where failures indicate bugs: `.unwrap()` with comments explaining why
- Match patterns to handle failures gracefully:
```rust
// From calibration.rs
let theta = match solve_theta(eta, gamma, rho, input.theta_star, input.k_star) {
    Some(t) => t,
    None => return 1e10,  // Return penalty value for infeasible parameters
};
```
- Constraint violations handled by returning large penalty values (1e10, 1e-10) rather than explicit error types
- Early returns with `?` operator for `Option` unwrapping: `calibrate()` uses `calibrate(&input, &config)?;`

## Logging

**Framework:** `println!` macros for output; no structured logging framework

**Patterns:**
- Informational output in binary/test code: `println!()` and `eprintln!()` not present (uses `println!` only)
- Report generation code prints detailed diagnostics:
```rust
// From tests/steep_skew.rs
println!("\n==========================================================");
println!("  {} (T = {})", label, t_expiry);
println!("==========================================================");
println!("  theta_star (ATM w) = {:.6e}", theta_star);
```
- Formatted output with alignment specifiers: `{:>8}`, `{:>10}`, `{:+10.6}`, `{:.6e}`, `{:.6}`
- No logging in library code (`src/lib.rs`, `src/ssvi.rs`, etc.)

## Comments

**When to Comment:**
- Mathematical formulas documented with inline comments explaining notation
- Complex algorithms preceded by explanation of steps
- Non-obvious constraints explained: "No sign change — return midpoint as best guess"
- Docstring-style comments for module-level documentation in some files

**JSDoc/TSDoc:**
- Not applicable (Rust project)
- Documentation comments use `///` for public APIs:
```rust
/// SSVI model: φ function and total variance w(k, θ).
/// φ(θ) = η / (θ^γ · (1+θ)^(1-γ))
#[inline]
pub fn phi(theta: f64, eta: f64, gamma: f64) -> f64 { ... }

/// Solve θ from the implicit ATM consistency equation...
pub fn solve_theta(...) -> Option<f64> { ... }
```
- No inline `//` comments in mathematical code; formula documentation uses `///` blocks

## Function Design

**Size:**
- Typically 10-50 lines for utility functions
- Larger functions (100+ lines) for complex algorithms like `nelder_mead_bounded()` (195 lines) and `brent()` (99 lines)
- Small, focused optimization objectives defined as closures within functions:
```rust
let objective_2d = |x: &[f64]| -> f64 {
    let eta = x[0];
    let gamma = x[1];
    // ... computation
};
```

**Parameters:**
- All numeric computations pass f64 directly
- Complex inputs wrapped in struct references: `&CalibrationInput`, `&NelderMeadConfig`
- Slices for array data: `&[f64]` for k_slice and w_market
- Closures for objective functions in optimizers: `F: Fn(&[f64]) -> f64` trait bounds

**Return Values:**
- `Option<T>` for potentially-failing operations
- Struct return types for composite results: `NelderMeadResult`, `CalibrationResult`, `BrentResult`
- Direct numeric returns for simple computations: `f64`
- Vectors for batch operations: `Vec<f64>` from `total_variance_slice()`

## Module Design

**Exports:**
- Public modules declared at crate root in `src/lib.rs`: `pub mod ssvi;`, `pub mod calibration;`, `pub mod nelder_mead;`, `pub mod brent;`
- Specific types and functions re-exported or accessed via module path: `essvi::calibration::calibrate()`, `essvi::ssvi::phi()`
- Private functions use no visibility modifier (default private in Rust)

**Barrel Files:**
- Not used; `src/lib.rs` is minimal with only module declarations
- Each module is directly importable: `use essvi::ssvi::*;` or `use essvi::calibration::CalibrationInput;`

**File Structure:**
- One major concept per file: `ssvi.rs` contains SSVI formulas, `calibration.rs` contains calibration logic, `brent.rs` contains root-finding, `nelder_mead.rs` contains optimization
- Tests colocated in modules using `#[cfg(test)] mod tests { ... }`
- Integration tests in separate `tests/` directory for end-to-end scenarios

---

*Convention analysis: 2026-03-05*
