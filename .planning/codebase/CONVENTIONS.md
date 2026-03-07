# Coding Conventions

**Analysis Date:** 2026-03-07

## Naming Patterns

**Files:**
- Use `snake_case.rs` for all source files: `nelder_mead.rs`, `fit_real.rs`, `steep_skew.rs`
- Library modules: descriptive nouns (`ssvi`, `calibration`, `brent`, `nelder_mead`)
- Binary entry points: verb-noun or descriptive (`report`, `fit_real`, `fit_real_surface`)
- Test files: descriptive scenario names (`steep_skew.rs`)
- Bench files: match the module being benchmarked (`calibration.rs`)

**Functions:**
- Use `snake_case` for all functions: `solve_theta`, `total_variance`, `no_arbitrage_satisfied`
- Pure math functions use short, domain-specific names: `phi`, `brent`, `project`
- Constructor/factory functions use `make_` prefix: `make_sample_slice`, `make_market_data`, `make_steep_skew_slice`, `make_20pt_slice`
- Fit/run functions use verb prefix: `run_scenario`, `fit_slice`, `run_calibration`
- Plot functions use `plot_` prefix: `plot_fit`, `plot_heatmap`

**Variables:**
- Mathematical variables follow paper notation: `eta`, `gamma`, `rho`, `theta`, `phi`, `k`, `w`
- Compound math names use underscore: `theta_star`, `k_star`, `k_slice`, `w_market`, `w_fit`
- Bounds use `lb`/`ub` prefix: `lb_eg`, `ub_eg`, `lb_3d`, `ub_3d`
- Loop/temp variables: short names (`s`, `t`, `p`, `pk`, `fr`, `dk`)
- Error metrics use descriptive names: `max_iv_err`, `rmse_iv`, `f_spread`, `x_spread`
- Counters: `n_rho`, `n_dense`, `n_t`, `n_s`

**Types/Structs:**
- Use `PascalCase`: `CalibrationInput`, `CalibrationResult`, `NelderMeadConfig`, `NelderMeadResult`, `BrentResult`
- Structs pair as Input/Result or Config/Result: `CalibrationInput`/`CalibrationResult`, `NelderMeadConfig`/`NelderMeadResult`
- Binary-only structs: `Scenario`, `FitResult`, `SliceData`

**Constants:**
- No named constants; numeric literals inline with comments explaining their meaning
- Tolerances: `1e-12`, `1e-14`, `1e-15`, `1e-30`
- Penalty values: `1e10` for infeasible objective returns
- Bounds: `1e-6`, `2.0 - 1e-6`, `0.999`

## Code Style

**Formatting:**
- Default `rustfmt` settings (no `.rustfmt.toml` present)
- 4-space indentation
- Trailing commas in struct literals and function arguments
- Run `cargo fmt` before committing

**Linting:**
- No `.clippy.toml` present; use default `cargo clippy` rules
- No `#[allow(...)]` attributes used anywhere in the codebase

**Edition:**
- Rust 2024 edition (`edition = "2024"` in `Cargo.toml`)

## Import Organization

**Order:**
1. `crate::` imports (internal modules)
2. External crate imports (`plotters`, `criterion`)
3. `std::` imports

**Example from `src/bin/report.rs`:**
```rust
use essvi::calibration::{calibrate, CalibrationInput};
use essvi::nelder_mead::NelderMeadConfig;
use essvi::ssvi;
use plotters::prelude::*;
use std::fs;
use std::io::Write;
```

**Patterns:**
- Import specific items from modules: `use crate::nelder_mead::{nelder_mead_bounded, NelderMeadConfig, NelderMeadResult}`
- Import module for namespaced access when many items used: `use crate::ssvi` then call `ssvi::phi(...)`, `ssvi::total_variance(...)`
- Glob imports only for `plotters::prelude::*`
- No path aliases configured

## Error Handling

**Library code (`src/`):**
- Use `Option<T>` for computations that may fail: `solve_theta` returns `Option<f64>`, `calibrate` returns `Option<CalibrationResult>`
- Return `None` for numerical failures (non-convergence, negative theta, zero derivative)
- No `Result<T, E>` types in the library; no custom error types defined
- Infeasible optimizer evaluations return a large penalty value (`1e10`) rather than failing

**Pattern for fallible math:**
```rust
// From src/calibration.rs
if theta <= 0.0 {
    return None;
}
// ...
if dw.abs() < 1e-30 {
    return None;
}
```

**Binary code (`src/bin/`):**
- Use `.expect("message")` for setup operations that must succeed
- Use `match` on `Option` for calibration results, with `eprintln!` for failures
- Use `Box<dyn std::error::Error>` return type for plot functions
- Use `?` operator for chaining plotters operations

**Pattern in binaries:**
```rust
// From src/bin/fit_real.rs
match fit_slice(slice) {
    Some(r) => { /* process */ }
    None => {
        eprintln!("Calibration FAILED for T={}", slice.t_expiry);
    }
}
```

## Logging

**Framework:** `println!` / `eprintln!` (no logging crate)

**Patterns:**
- `println!` for progress output and results in binaries
- `eprintln!` for error conditions in binaries
- No logging in library code (`src/ssvi.rs`, `src/calibration.rs`, `src/nelder_mead.rs`, `src/brent.rs`)
- Formatted numeric output uses `{:.Nf}`, `{:.Ne}`, `{:>N}` alignment specifiers

## Comments

**When to Comment:**
- Every public function gets a `///` doc comment explaining its mathematical purpose
- Mathematical equations rendered in comments using Unicode symbols and ASCII math notation
- Section separators in binaries use `// -- Section Name --` with em-dash box-drawing characters

**Doc Comment Style:**
```rust
/// SSVI total variance for a single strike:
/// w(k, theta) = (theta/2) * {1 + rho*phi(theta)*k + sqrt((phi(theta)*k + rho)^2 + (1 - rho^2))}
#[inline]
pub fn total_variance(k: f64, theta: f64, eta: f64, gamma: f64, rho: f64) -> f64 {
```

**Section markers in binaries:**
```rust
// -- Scenario parameters --
// -- Plot generation --
// -- Main --
```

**Test doc comments:** Tests use `///` comments to explain what the test verifies, especially for non-obvious mathematical properties.

## Function Design

**Size:**
- Library functions are small and focused (5-30 lines typically)
- Binary `main()` functions are longer (50-100 lines), orchestrating the full pipeline
- Helper functions extracted for repeated patterns (`make_market_data`, `run_scenario`, `compute_fit_result`)

**Parameters:**
- Use borrowed slices (`&[f64]`) for input data arrays
- Use struct references for grouped parameters: `&CalibrationInput`, `&NelderMeadConfig`
- Use `Option<&[f64]>` for optional parameters (e.g., `weights`)
- Lifetime annotations with `'a` on input structs that borrow data: `CalibrationInput<'a>`

**Return Values:**
- `Option<T>` for fallible operations
- `Vec<f64>` for computed arrays
- Named result structs for multi-value returns: `CalibrationResult`, `NelderMeadResult`, `BrentResult`
- `Result<(), Box<dyn std::error::Error>>` for I/O operations in binaries

**Performance Annotations:**
- `#[inline]` on small, hot math functions: `phi`, `total_variance`, `no_arbitrage_satisfied`, `project`
- Closures for optimizer objectives (captured by reference)

## Module Design

**Exports:**
- All library modules declared as `pub mod` in `src/lib.rs`
- Public functions and structs use `pub` visibility
- Internal helper functions are private (e.g., `weighted_squared_error`, `calendar_penalty`, `project`)
- No re-exports or facade patterns

**Barrel Files:**
- `src/lib.rs` serves as the barrel file, exposing all four modules
- No nested re-exports; consumers import from specific modules: `essvi::calibration::calibrate`

**Module Boundaries:**
- `ssvi` - pure math (SSVI formulas), no dependencies
- `brent` - generic root-finding algorithm, no dependencies
- `nelder_mead` - generic optimizer, no dependencies
- `calibration` - orchestration layer, depends on `ssvi` and `nelder_mead`

## Numeric Conventions

**Tolerances:**
- Convergence tolerance: `1e-12` (Nelder-Mead default for both `tol_f` and `tol_x`)
- Newton method tolerance: `1e-14` (tight for `solve_theta`)
- Near-zero guard: `1e-15` for `k_star`, `1e-30` for derivatives
- Post-convergence relaxed check: `tol * 100.0`

**Infeasible penalty:** Return `1e10` when constraints are violated inside an optimizer objective.

**Parameter bounds:**
- `eta` in `[1e-6, 2.0 - 1e-6]`
- `gamma` in `[1e-6, 1.0 - 1e-6]`
- `rho` in `[-0.999, 0.999]`

---

*Convention analysis: 2026-03-07*
