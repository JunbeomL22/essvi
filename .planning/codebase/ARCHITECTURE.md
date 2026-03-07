# Architecture

**Analysis Date:** 2026-03-08

## Pattern Overview

**Overall:** Domain-specific numerical library with CLI binaries

This is a Rust library crate (`essvi`) implementing the Surface SVI (SSVI) implied volatility model for options pricing. The architecture follows a layered numerical computing pattern: low-level math primitives feed into pricing models, which feed into calibration solvers, which are consumed by CLI binaries.

**Key Characteristics:**
- Pure Rust, zero external runtime dependencies (only `plotters` for visualization)
- No async, no networking, no database -- pure numerical computation
- Library crate (`src/lib.rs`) exposes all modules; binaries in `src/bin/` consume the library
- Two calibration approaches: implicit-theta Newton solver (`calibration`) and direct 4D Nelder-Mead (`crude_solver`)
- No-arbitrage constraints enforced either by construction or via penalty terms

## Layers

**Math Layer:**
- Purpose: Machine-precision implementations of fundamental mathematical functions
- Location: `src/math/`
- Contains: Error function (`erf.rs`), normal distribution PDF/CDF/inverse (`normal.rs`, `normal_hp.rs`), floating-point constants (`constants.rs`)
- Depends on: Nothing (leaf layer)
- Used by: Pricing layer

**Pricing Layer:**
- Purpose: Black-76 option pricing and implied volatility extraction
- Location: `src/pricing/`
- Contains: Black-76 price/greeks (`black76.rs`), Let's Be Rational implied vol solver (`lets_be_rational.rs`), rational cubic interpolation (`rational_cubic.rs`), error types (`error.rs`)
- Depends on: Math layer
- Used by: Binaries (for IV conversion in data pipelines)

**Model Layer:**
- Purpose: SSVI parametric model definition
- Location: `src/model/`
- Contains: `ssvi.rs` -- phi function, total variance formula, no-arbitrage check, slice evaluation
- Depends on: Nothing (pure math formulas)
- Used by: Calibration layer, crude solver, binaries

**Solver Layer:**
- Purpose: Generic numerical optimization algorithms
- Location: `src/solver/`
- Contains: Bounded Nelder-Mead simplex optimizer (`nelder_mead.rs`), Brent's root-finding method (`brent.rs`)
- Depends on: Nothing (generic algorithms)
- Used by: Calibration layer, crude solver

**Calibration Layer (Implicit Theta):**
- Purpose: SSVI calibration via implicit theta solve + rho-grid + 2D/3D Nelder-Mead
- Location: `src/calibration.rs`
- Contains: `CalibrationConfig`, `CalibrationInput`, `CalibrationResult`, `CalibError`, `solve_theta`, `calibrate`, `calibrate_with_calendar_penalty`, `PrevSlice`
- Depends on: Model layer, Solver layer
- Used by: Binaries (`fit_real`, `fit_real_surface`, `fit_cboe`, `fit_kaggle`, `report`)

**Crude Solver Layer (Direct 4D):**
- Purpose: Alternative calibration treating all 4 SSVI parameters as free variables
- Location: `src/crude_solver.rs`
- Contains: `CrudeCalibConfig`, `CrudeCalibResult`, `SliceInput`, `calibrate_slice`, `calibrate_surface`, butterfly density/penalty functions
- Depends on: Model layer, Solver layer
- Used by: Binary (`fit_crude`)

**Shared Fit Utilities:**
- Purpose: Common types and plotting for fit binaries
- Location: `src/fit_common.rs`
- Contains: `SliceData`, `FitResult`, `make_slice`, `build_market_slices`, `plot_fit`
- Depends on: Model layer, `plotters`
- Used by: All fit binaries

**Binary Layer:**
- Purpose: CLI executables for calibration, reporting, and visualization
- Location: `src/bin/`
- Contains: `fit_real.rs`, `fit_real_surface.rs`, `fit_cboe.rs`, `fit_kaggle.rs`, `fit_crude.rs`, `plot_kaggle.rs`, `report.rs`
- Depends on: All library layers
- Used by: End users via `cargo run --bin <name>`

## Data Flow

**Single-Slice Calibration (Implicit Theta):**

1. Market IV data (k, sigma) converted to total variance: w = sigma^2 * T
2. ATM reference point identified: theta_star = sigma_atm^2 * T, k_star = k_atm
3. Rho swept on grid (-0.95 to 0.95, 21 points)
4. For each rho: 2D Nelder-Mead optimizes (eta, gamma) with objective function:
   - Inside objective: `solve_theta` via Newton's method eliminates theta as free variable
   - SSVI total variance computed for all strikes
   - Weighted squared error returned
5. Best (eta, gamma, rho) from grid used as starting point for 3D polish
6. Final theta solved from best (eta, gamma, rho)
7. `CalibrationResult` returned with parameters + optimizer diagnostics

**Single-Slice Calibration (Crude Solver):**

1. Market IV data (k, sigma) converted to total variance: w = sigma^2 * T
2. Multi-start grid: 4 rho starts x 3 eta starts = 12 starting points
3. Each start: 4D Nelder-Mead optimizes (theta, eta, gamma, rho) directly
4. Objective = SSE + lambda * butterfly_penalty (+ optional calendar penalty)
5. No-arbitrage hard barrier: eta*(1+|rho|) <= 2 returns 1e10 penalty
6. Best result across all starts selected

**Surface Calibration (Sequential):**

1. Slices sorted by expiry T ascending
2. First slice calibrated without calendar constraint
3. Each subsequent slice calibrated with calendar penalty:
   - `calibration.rs`: lambda * sum(max(0, w_prev(k) - w_cur(k))^2) at penalty sample points
   - `crude_solver.rs`: lambda_calendar * max(0, prev_theta - theta)^2 for theta monotonicity
4. Previous slice parameters passed as `PrevSlice` / `prev_theta`

**CBOE Data Pipeline (fit_cboe / fit_crude):**

1. CSV parsed: rows grouped by DTE, 3-zone IV rule applied (p_iv / mean / c_iv by log_moneyness)
2. Slices with < 5 points filtered out; log_moneyness restricted to [-1.0, 1.0]
3. Surface calibrated with calendar penalty
4. SVG plots generated per slice; markdown report written

**State Management:**
- No persistent state. All binaries are stateless CLI tools that read CSV files, calibrate, and write output files (SVG plots, markdown reports).
- Calibration config structs (`CalibrationConfig`, `CrudeCalibConfig`) use `Default` trait for sensible defaults.

## Key Abstractions

**SSVI Model (`src/model/ssvi.rs`):**
- Purpose: The mathematical SSVI parameterization of the volatility surface
- Pattern: Pure functions, no state. `phi(theta, eta, gamma)` and `total_variance(k, theta, eta, gamma, rho)`
- Key constraint: `no_arbitrage_satisfied(eta, rho)` enforces eta*(1+|rho|) <= 2

**CalibrationInput / CalibrationResult (`src/calibration.rs`):**
- Purpose: Input/output data types for the implicit-theta calibration pipeline
- Pattern: Borrowed input (`CalibrationInput<'a>` with slice references), owned output (`CalibrationResult`)
- `CalibrationInput` carries: k_slice, w_market, theta_star, k_star, optional weights
- `CalibrationResult` carries: eta, gamma, rho, theta, optimizer result

**CrudeCalibConfig / CrudeCalibResult (`src/crude_solver.rs`):**
- Purpose: Config and result for direct 4D calibration
- Pattern: Config struct with `Default` impl for all tuning knobs
- `CrudeCalibResult` carries: theta, eta, gamma, rho, sse, converged, iterations

**FitResult (`src/fit_common.rs`):**
- Purpose: Unified result type for all fit binaries, suitable for plotting and reporting
- Pattern: Carries both calibrated parameters and error metrics (max_iv_err_bps, rmse_iv_bps) plus IV vectors for plotting
- Used by both calibration approaches via manual construction in binaries

**NelderMeadConfig / NelderMeadResult (`src/solver/nelder_mead.rs`):**
- Purpose: Generic bounded derivative-free optimizer
- Pattern: Higher-order function -- accepts closure `F: Fn(&[f64]) -> f64` as objective
- Works in arbitrary dimensions (2D, 3D, 4D in this codebase)

## Entry Points

**Library Entry (`src/lib.rs`):**
- Location: `src/lib.rs`
- Exposes: `calibration`, `crude_solver`, `fit_common`, `math`, `model`, `pricing`, `solver`
- Re-exports: `model::ssvi`, `solver::brent`, `solver::nelder_mead` for backward compatibility

**fit_cboe (`src/bin/fit_cboe.rs`):**
- Usage: `cargo run --bin fit_cboe -- <ticker> <date>`
- Triggers: CLI args (e.g., `spx 2026-03-07`)
- Responsibilities: Parse CBOE CSV, fit surface with implicit-theta solver, generate SVG plots + markdown report

**fit_crude (`src/bin/fit_crude.rs`):**
- Usage: `cargo run --bin fit_crude -- <ticker> <date>`
- Triggers: CLI args
- Responsibilities: Parse CBOE CSV, fit surface with crude 4D solver, generate SVG plots + summary

**fit_kaggle (`src/bin/fit_kaggle.rs`):**
- Usage: `cargo run --bin fit_kaggle -- <date>`
- Triggers: CLI args (e.g., `2020-03-13`)
- Responsibilities: Parse Kaggle SPY CSV, fit surface, generate plots + report

**fit_real (`src/bin/fit_real.rs`):**
- Triggers: `cargo run --bin fit_real`
- Responsibilities: Fit SSVI to synthetic market data (12 slices), generate plots + report

**fit_real_surface (`src/bin/fit_real_surface.rs`):**
- Triggers: `cargo run --bin fit_real_surface`
- Responsibilities: Surface fit with calendar penalty on synthetic data

**report (`src/bin/report.rs`):**
- Triggers: `cargo run --bin report`
- Responsibilities: Generate parameter grid quality report with heatmaps

**plot_kaggle (`src/bin/plot_kaggle.rs`):**
- Triggers: `cargo run --bin plot_kaggle`
- Responsibilities: Plot raw IV smiles from Kaggle data (no fitting)

## Error Handling

**Strategy:** Typed error enums at each layer, `Result` propagation

**Patterns:**
- `CalibError` (4 variants): `NonPositiveTheta`, `ZeroDerivative`, `ThetaDivergence`, `NonConvergence` -- covers Newton solver and optimizer failures
- `PricingError` (3 variants): `AboveMaximum`, `BelowIntrinsic`, `InvalidInput` -- covers invalid inputs and out-of-range prices
- Both implement `std::error::Error` + `Display`
- Crude solver returns `CrudeCalibResult` directly (no Result wrapper) since it always returns a best-effort answer
- Binaries use `.ok()?`, `.expect()`, or `match` with `eprintln!` for error reporting
- No custom error types for math layer -- functions are total (always return valid f64)

## Cross-Cutting Concerns

**Logging:** `println!` / `eprintln!` in binaries only. No logging framework. Library code is silent.

**Validation:** Input validation in pricing layer (`validate_inputs`). Calibration uses hard-barrier penalties (return 1e10) for constraint violations rather than upfront validation.

**Authentication:** Not applicable -- standalone numerical library with no network access.

**Visualization:** `plotters` crate used exclusively for SVG output. Plot generation is in `src/fit_common.rs::plot_fit` (shared) and `src/bin/report.rs::plot_fit` / `plot_heatmap` (local).

---

*Architecture analysis: 2026-03-08*
