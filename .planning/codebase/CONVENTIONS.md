# Coding Conventions

**Analysis Date:** 2026-03-08

## Naming Patterns

**Files:**
- Use `snake_case.rs` for all source files: `crude_solver.rs`, `nelder_mead.rs`, `lets_be_rational.rs`
- Binary entry points prefixed by purpose: `fit_real.rs`, `fit_cboe.rs`, `plot_kaggle.rs`
- Module index files use `mod.rs`

**Functions:**
- Use `snake_case` for all functions: `solve_theta`, `calibrate_slice`, `normalised_black_call`
- Constructors/builders not used; prefer free functions over methods
- Mathematical operations keep short names matching the formulas: `phi()`, `d1_d2()`, `project()`

**Variables:**
- Use short mathematical variable names matching the underlying formulas: `eta`, `gamma`, `rho`, `theta`, `k`, `w`, `t`, `s`
- Prefix `true_` for known ground-truth values in tests: `true_eta`, `true_gamma`
- Suffix `_star` for ATM reference values: `theta_star`, `k_star`
- Suffix `_slice` or `_market` for data arrays: `k_slice`, `w_market`, `iv_fit`
- Bounds use `_lower` / `_upper` suffixes: `eta_lower`, `eta_upper`

**Types:**
- Use `PascalCase` for structs and enums: `CalibrationConfig`, `CalibError`, `PricingError`
- Result structs suffixed with `Result`: `CalibrationResult`, `NelderMeadResult`, `BrentResult`, `CrudeCalibResult`
- Config structs suffixed with `Config`: `CalibrationConfig`, `NelderMeadConfig`, `CrudeCalibConfig`
- Input structs suffixed with `Input`: `CalibrationInput`, `SliceInput`

**Constants:**
- Use `SCREAMING_SNAKE_CASE`: `DBL_EPSILON`, `SQRT_TWO_PI`, `ONE_OVER_SQRT_TWO`
- Algorithm-specific constants use descriptive prefixes: `ERF_SMALL_THRESHOLD`, `NORM_CDF_ASYMPTOTIC_EXPANSION_FIRST_THRESHOLD`
- Polynomial coefficient arrays are named with uppercase abbreviation + index: `PP0`, `PP1`, `PA0`, `RA0`, `QQ1`

## Code Style

**Formatting:**
- No `.rustfmt.toml` present; uses default `rustfmt` settings
- Run `cargo fmt` before committing

**Linting:**
- No `.clippy.toml` present; uses default `clippy` settings
- Run `cargo clippy` for lint checks

**Indentation:**
- 4 spaces (Rust default)

**Line Length:**
- No explicit limit; lines up to ~100 characters are common, some mathematical expressions extend further

**Trailing Commas:**
- Used in struct definitions and multi-line function calls (Rust convention)

## Import Organization

**Order:**
1. Crate-internal imports (`use crate::...`)
2. Standard library imports (`use std::...`)
3. External crate imports (`use plotters::...`, `use criterion::...`)

**Path Aliases:**
- No path aliases configured; all imports use full crate paths
- Re-exports in `src/lib.rs` for backward compatibility:
  ```rust
  pub use model::ssvi;
  pub use solver::brent;
  pub use solver::nelder_mead;
  ```

**Import Style:**
- Import specific items, not glob imports: `use crate::model::ssvi;` then call `ssvi::phi()`
- Nested imports when multiple items from same module:
  ```rust
  use crate::solver::nelder_mead::{NelderMeadConfig, NelderMeadResult, nelder_mead_bounded};
  ```

## Module Organization

**Module declarations** in `mod.rs` files are minimal -- just `pub mod` statements:
```rust
// src/solver/mod.rs
pub mod brent;
pub mod nelder_mead;
```

**Module-level doc comments** use `///` on the first line of each file:
```rust
/// Bounded Nelder-Mead optimizer (derivative-free).
```

**Re-exports** are used in `src/lib.rs` to maintain backward-compatible import paths.

## Documentation Patterns

**Doc comments (`///`):**
- Required on all public functions, structs, enums, and their fields
- Mathematical formulas written in plain text with Unicode symbols: `φ(θ) = η / (θ^γ · (1+θ)^(1-γ))`
- Include derivation steps for numerical algorithms (see `src/calibration.rs` lines 102-112)
- Use `# Arguments`, `# Returns`, `# Examples` sections on key public functions

**Doc examples:**
- Present on all public pricing functions in `src/pricing/black76.rs`
- Present on all public math functions in `src/math/normal.rs`, `src/math/erf.rs`
- Present on public implied vol functions in `src/pricing/lets_be_rational.rs`
- Examples are run as doctests (20 doctests total)
- Pattern:
  ```rust
  /// # Examples
  /// ```
  /// # use essvi::pricing::black76::price;
  /// let c = price(100.0, 100.0, 0.20, 1.0, 1).unwrap();
  /// assert!((c - 7.965567455405804).abs() < 1e-10);
  /// ```
  ```

**Inline comments:**
- Explain mathematical derivation steps inline: `// ∂w/∂θ`
- Section dividers using `// ── Section Name ──────────...`
- Comment non-obvious numerical choices: `// |x| < 2^-28: erf(x) ~ x * (2/sqrt(pi))`

**Field-level documentation:**
- All config struct fields have `///` doc comments explaining semantics and constraints:
  ```rust
  /// Lower bound for η (must be > 0).
  pub eta_lower: f64,
  ```

## Error Handling

**Patterns:**
- Use `Result<T, E>` with domain-specific error enums for fallible operations
- Two custom error types: `CalibError` (`src/calibration.rs`) and `PricingError` (`src/pricing/error.rs`)
- All error types implement `fmt::Display` and `std::error::Error`
- Error variants carry structured data for diagnostics:
  ```rust
  AboveMaximum { price: f64, maximum: f64 },
  BelowIntrinsic { price: f64, intrinsic: f64 },
  InvalidInput(String),
  ```

**Error propagation:**
- Use `?` operator for propagation within the same error domain
- Use `1e10` sentinel returns (not `Err`) inside closure-based objectives where errors cannot propagate:
  ```rust
  let theta = match solve_theta(...) {
      Ok(t) => t,
      Err(_) => return 1e10,  // penalty for infeasible parameters
  };
  ```

**Validation:**
- Input validation at function boundaries via dedicated `validate_inputs()` helper (`src/pricing/black76.rs`)
- Bounds checking with early returns: `if forward <= 0.0 { return Err(...); }`
- No-arbitrage condition checked with `ssvi::no_arbitrage_satisfied()` before optimization

**Panics:**
- Avoided in library code; all errors returned via `Result`
- `expect()` used only in binary entry points and test setup: `res.expect("at least one start point must run")`
- `.unwrap()` used only in tests and binaries, never in library code

## Function Design

**Size:**
- Functions generally kept under 60 lines
- Larger functions (like `calibrate`) decompose into helper closures and sub-functions
- Mathematical algorithms allowed to be longer when the algorithm is a single logical unit

**Parameters:**
- Config structs collect tuning knobs: pass `&CalibrationConfig` instead of individual parameters
- Input data bundled into input structs: `CalibrationInput`, `SliceInput`
- Slice references `&[f64]` used for numerical arrays; owned `Vec<f64>` for results
- Lifetimes used on input structs: `CalibrationInput<'a>` with `pub k_slice: &'a [f64]`

**Return Values:**
- `Result<T, E>` for operations that can fail
- Plain structs for infallible operations: `nelder_mead_bounded` returns `NelderMeadResult` (no `Result` wrapper)
- `Vec<f64>` for computed arrays; individual `f64` for scalar computations
- `Option` used sparingly; prefer `Result` with meaningful errors

## Performance Annotations

**Inlining:**
- `#[inline]` on small, hot mathematical functions: `phi()`, `total_variance()`, `norm_pdf()`, `norm_cdf()`, `d1_d2()`, `project()`
- Not used on larger functions or closure-heavy calibration code

**Derive macros:**
- `#[derive(Debug, Clone)]` on all public structs and enums
- `#[derive(Debug, Clone, Copy)]` on small value types: `Greeks`
- `Default` implemented manually (not derived) for config structs with non-trivial defaults

## Numerical Conventions

**Tolerances:**
- Machine epsilon constants defined in `src/math/constants.rs`; use these instead of magic numbers
- Newton solver tolerances: `1e-14` for convergence, `1e-30` for near-zero derivative detection
- Optimizer defaults: `tol_f = 1e-12`, `tol_x = 1e-12`
- Assertion tolerances in tests: `1e-15` for exact math, `1e-10` for round-trip, `1e-3` for optimization recovery

**f64 arithmetic:**
- Use `.powi(2)` instead of `x * x` for squaring in non-critical paths
- Use `x * x` explicitly in hot numerical loops for clarity
- Use `.clamp(lo, hi)` for bound enforcement
- Use `f64::INFINITY` and `f64::NEG_INFINITY` for initial best values in search loops

**Option type call/put flag:**
- Convention `q = +1` for call, `q = -1` for put (following Let's Be Rational convention)
- Type is `i32`, not an enum; validated at function boundaries

## Binary Conventions

**Binary files** in `src/bin/` follow a common pattern:
- Parse data from CSV or generate synthetic data
- Call library calibration functions
- Write results to `documents/` directory (markdown reports + SVG plots)
- Use `plotters` crate for SVG chart generation
- `main()` function handles I/O; calibration logic delegated to library

**Shared binary helpers** in `src/fit_common.rs`:
- `SliceData`, `FitResult` types shared across binaries
- `build_market_slices()` for synthetic data generation
- `plot_fit()` for SVG rendering

---

*Convention analysis: 2026-03-08*
