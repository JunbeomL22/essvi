# Architecture

**Analysis Date:** 2026-03-07

## Pattern Overview

**Overall:** Domain-focused numerical library with layered computation pipeline

**Key Characteristics:**
- Pure Rust library crate (`lib.rs`) exposing four public modules for SSVI volatility surface calibration
- Computation pipeline: SSVI model evaluation -> implicit theta solving -> derivative-free optimization -> calibration orchestration
- Binary targets serve as report generators and demonstration tools, not as the primary interface
- No external dependencies for core logic; `plotters` used only in binary targets for SVG output
- All numerical algorithms implemented from scratch (Nelder-Mead, Brent's method) to maintain zero-dependency core

## Layers

**Model Layer (SSVI Formulas):**
- Purpose: Evaluate the SSVI total variance formula and check no-arbitrage conditions
- Location: `src/ssvi.rs`
- Contains: `phi()`, `total_variance()`, `total_variance_slice()`, `no_arbitrage_satisfied()`
- Depends on: Nothing (leaf module, pure math)
- Used by: `src/calibration.rs`, all binaries, all tests

**Numerical Solvers Layer:**
- Purpose: Provide general-purpose numerical optimization and root-finding primitives
- Location: `src/nelder_mead.rs`, `src/brent.rs`
- Contains:
  - `nelder_mead_bounded()` - Bounded Nelder-Mead simplex optimizer (derivative-free)
  - `brent()` - Brent's method root finder (currently unused in production code but available)
  - `NelderMeadConfig`, `NelderMeadResult`, `BrentResult` - Configuration and result structs
- Depends on: Nothing (generic numerical algorithms)
- Used by: `src/calibration.rs`

**Calibration Layer:**
- Purpose: Orchestrate SSVI parameter fitting against market data
- Location: `src/calibration.rs`
- Contains:
  - `solve_theta()` - Newton's method solver for the implicit ATM consistency equation
  - `calibrate()` - Full per-slice calibration (rho-grid sweep + 2D Nelder-Mead + 3D polish)
  - `calibrate_with_calendar_penalty()` - Surface calibration with cross-slice consistency penalty
  - `CalibrationInput`, `CalibrationResult`, `PrevSlice` - Data structures
  - `weighted_squared_error()` - Internal objective function helper
- Depends on: `src/ssvi.rs`, `src/nelder_mead.rs`
- Used by: All binary targets, integration tests, benchmarks

**Binary Layer (Report Generators):**
- Purpose: Generate fit quality reports with SVG plots and markdown output
- Location: `src/bin/report.rs`, `src/bin/fit_real.rs`, `src/bin/fit_real_surface.rs`
- Contains: Synthetic market data generation, calibration invocation, SVG plotting, markdown report writing
- Depends on: All library modules + `plotters` crate
- Used by: Developer (run manually for analysis)

## Data Flow

**Single-Slice Calibration (`calibrate()`):**

1. Caller provides `CalibrationInput` containing log-moneyness slice `k_slice`, market total variances `w_market`, ATM reference point `(theta_star, k_star)`, and optional `weights`
2. Outer loop sweeps rho from -0.95 to 0.95 in 21 steps
3. For each rho, a 2D Nelder-Mead optimizes `(eta, gamma)` where the objective:
   a. Checks no-arbitrage: `eta * (1 + |rho|) <= 2`
   b. Solves theta implicitly via Newton iteration in `solve_theta()` (the ATM consistency equation `w(k*, theta) = theta*`)
   c. Evaluates `weighted_squared_error()` between model and market total variances
4. Best `(eta, gamma, rho)` from grid is polished with a 3D Nelder-Mead optimization
5. Final `CalibrationResult` returned with fitted `(eta, gamma, rho, theta)` and optimizer diagnostics

**Surface Calibration (`calibrate_with_calendar_penalty()`):**

1. Slices are processed sequentially from shortest to longest expiry
2. First slice uses unconstrained `calibrate()`
3. Subsequent slices add a penalty term: `lambda * sum(max(0, w_prev(k) - w_cur(k))^2)` evaluated at sample k-points
4. Initial guess comes from unconstrained per-slice fit
5. Calendar arbitrage penalty enforces that total variance is non-decreasing in expiry at each k

**State Management:**
- No mutable global state; all computation is purely functional
- `CalibrationInput` borrows market data (lifetime `'a`) to avoid copies
- Nelder-Mead simplex is heap-allocated internally and returned as `Vec<f64>` in results

## Key Abstractions

**CalibrationInput:**
- Purpose: Encapsulates all market data needed for a single slice calibration
- Examples: `src/calibration.rs` line 90-96
- Pattern: Borrow-based struct with lifetime parameter to avoid data copying

**NelderMeadConfig / NelderMeadResult:**
- Purpose: Configuration and output for the optimizer, decoupled from the specific optimization problem
- Examples: `src/nelder_mead.rs` lines 4-34
- Pattern: Builder-like with `Default` implementation for config; result struct captures convergence status

**CalibrationResult:**
- Purpose: Holds fitted SSVI parameters plus optimizer diagnostics
- Examples: `src/calibration.rs` lines 98-105
- Pattern: Flat struct containing both domain values (eta, gamma, rho, theta) and optimizer metadata

**PrevSlice:**
- Purpose: Carries fitted parameters from the previous expiry slice for calendar arbitrage penalty computation
- Examples: `src/calibration.rs` lines 186-192
- Pattern: Simple value struct, no references

## Entry Points

**Library Entry (`lib.rs`):**
- Location: `src/lib.rs`
- Triggers: `use essvi::{ssvi, calibration, nelder_mead, brent}`
- Responsibilities: Re-exports all four public modules

**`fit_real` Binary:**
- Location: `src/bin/fit_real.rs`
- Triggers: `cargo run --bin fit_real`
- Responsibilities: Per-slice SSVI calibration on 12 synthetic market slices (approximating real equity index data), generates SVG plots and markdown report to `documents/`

**`fit_real_surface` Binary:**
- Location: `src/bin/fit_real_surface.rs`
- Triggers: `cargo run --bin fit_real_surface`
- Responsibilities: Two-step surface calibration (unconstrained then penalized), generates SVG plots and markdown report with calendar arbitrage analysis

**`report` Binary:**
- Location: `src/bin/report.rs`
- Triggers: `cargo run --bin report`
- Responsibilities: Parameter grid sweep (T x slope), heatmap visualization, no-arbitrage constraint saturation analysis

## Error Handling

**Strategy:** `Option`-based propagation for numerical failures; no custom error types

**Patterns:**
- `solve_theta()` returns `Option<f64>` -- `None` when Newton iteration diverges or theta goes non-positive
- `calibrate()` and `calibrate_with_calendar_penalty()` return `Option<CalibrationResult>` -- `None` propagated from `solve_theta()` failure
- Inside the Nelder-Mead objective function, numerical failures return a large penalty value (`1e10`) rather than propagating errors, keeping the optimizer running
- Binary targets use `.expect()` / `unwrap()` for top-level calls, printing error messages for per-slice failures and continuing

## Cross-Cutting Concerns

**Logging:** Console output via `println!`/`eprintln!` in binaries only; library code is silent
**Validation:** No-arbitrage constraint checked inline within objective functions (`ssvi::no_arbitrage_satisfied()`); parameter bounds enforced via Nelder-Mead projection (`project()`)
**Authentication:** Not applicable (pure computation library)
**Serialization:** None; results are consumed in-process or written as markdown/SVG by binaries
**Performance:** `#[inline]` annotations on hot-path model evaluation functions (`phi`, `total_variance`, `no_arbitrage_satisfied`, `project`)

---

*Architecture analysis: 2026-03-07*
