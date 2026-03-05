# Architecture

**Analysis Date:** 2026-03-05

## Pattern Overview

**Overall:** Stochastic Volatility Inspired (SSVI) Volatility Model Library

This is a mathematical modeling library implementing the SSVI parametric smile model for financial derivatives pricing. The architecture follows a layered, composable design where low-level numerical algorithms combine to form higher-level volatility modeling and calibration functionality.

**Key Characteristics:**
- **Mathematical domain focus** — Models financial smile/skew across log-moneyness strikes
- **Algorithm composition** — Root-finding and optimization primitives form building blocks for model calibration
- **No-arbitrage constraints** — Enforces financial constraints during parameter optimization
- **Implicit equation solving** — Uses numerical methods to eliminate equality constraints
- **Grid-sweep + polishing** — Two-phase optimization to avoid local minima in high-dimensional parameter space

## Layers

**Core Mathematical Model:**
- Purpose: Implement the SSVI volatility parametrization equations
- Location: `src/ssvi.rs`
- Contains: Pure mathematical functions (phi, total_variance, no-arbitrage checking)
- Depends on: None (pure Rust stdlib)
- Used by: Calibration layer, reporting, tests

**Numerical Solvers:**
- Purpose: Provide root-finding and unconstrained optimization primitives
- Location: `src/brent.rs` (root-finding), `src/nelder_mead.rs` (function minimization)
- Contains: Brent's bracketed root-finding method, bounded Nelder-Mead simplex optimizer
- Depends on: None
- Used by: Calibration layer

**Calibration Engine:**
- Purpose: Solve SSVI parameters from market volatility data
- Location: `src/calibration.rs`
- Contains: `solve_theta()` for implicit ATM consistency, `calibrate()` for full parameter estimation
- Depends on: `src/brent.rs`, `src/nelder_mead.rs`, `src/ssvi.rs`
- Used by: Binary (reporting), tests, benchmarks

**Reporting & Visualization:**
- Purpose: Generate diagnostic plots and quality metrics across parameter grids
- Location: `src/bin/report.rs`
- Contains: Scenario generation, fit quality assessment, SVG plot generation using plotters
- Depends on: All library modules
- Used by: Command-line execution to create documentation

**Testing & Benchmarking:**
- Purpose: Validate correctness and measure performance
- Location: `tests/steep_skew.rs`, `benches/calibration.rs`
- Contains: Unit/integration tests in modules, criterion benchmarks
- Depends on: Library modules

## Data Flow

**Calibration Flow:**

1. **Input Stage** (`CalibrationInput`):
   - Market data: `k_slice` (log-moneyness points), `w_market` (total variances)
   - Constraints: `theta_star` (ATM total variance), `k_star` (ATM strike offset)

2. **Grid Search (Rho Sweep)**:
   - For each ρ value in [-0.95, 0.95] (20 points):
     - Inner optimization: Minimize squared error over (η, γ) using bounded Nelder-Mead
     - For each (η, γ, ρ) candidate:
       - Solve implicit equation θ = θ* / (1 + ρ·φ(θ)·k*) via Brent's method
       - Compute model total variance `w_model` at all k points
       - Calculate sum-of-squared-errors (SSE) vs market
     - Track best (η, γ, ρ) and corresponding SSE

3. **Polish Phase (3D Refinement)**:
   - Restart Nelder-Mead from best grid point
   - Optimize over full 3D space (η, γ, ρ) simultaneously
   - Solve θ again for final parameter set
   - Enforce no-arbitrage constraint: η·(1+|ρ|) ≤ 2

4. **Output** (`CalibrationResult`):
   - Calibrated parameters: η, γ, ρ, θ
   - Optimizer convergence info: iterations, final SSE, convergence flag

**State Management:**
- State is **immutable functional** — each function takes inputs and returns outputs
- No shared state between calibration runs
- Simplex vertices in Nelder-Mead algorithm maintained locally during optimization
- Brent's algorithm state (bracketing interval, function values) local to solver

## Key Abstractions

**SSVI Model:**
- Purpose: Represent and compute volatility smiles using parametric form
- Files: `src/ssvi.rs`
- Pattern: Pure functions operating on parameters → volatility quantities
- Key functions:
  - `phi(θ, η, γ)` — Shape parameter, controls smile curvature relative to skew
  - `total_variance(k, θ, η, γ, ρ)` — Model total variance at log-moneyness k
  - `no_arbitrage_satisfied(η, ρ)` — Constraint check

**Brent's Method:**
- Purpose: Root-finding on bracketed intervals [a, b]
- Files: `src/brent.rs`
- Pattern: Generic function accepting closure f, returns `BrentResult` with convergence info
- Used for: Solving implicit θ equation without closed-form solution

**Nelder-Mead Optimizer:**
- Purpose: Minimize function over bounded box [lb, ub]
- Files: `src/nelder_mead.rs`
- Pattern: Simplex-based derivative-free optimization, respects box constraints via projection
- Configuration: `NelderMeadConfig` with convergence tolerances, reflection/expansion coefficients
- Used for: Multi-dimensional parameter estimation in calibration

**Calibration Input/Result:**
- Purpose: Encapsulate calibration problem and solution
- Pattern: Borrowed references for input (no copies), owned data for output
- `CalibrationInput`: References to k_slice, w_market vectors + ATM constraint values
- `CalibrationResult`: Owned parameter values + optimizer diagnostics

## Entry Points

**Library Entry (`src/lib.rs`):**
- Location: `src/lib.rs`
- Triggers: Used when imported as crate (e.g., in tests, benchmarks, external code)
- Responsibilities: Module re-export (brent, calibration, nelder_mead, ssvi)
- Public API: All four modules available for external use

**Binary Entry (`src/bin/report.rs`):**
- Location: `src/bin/report.rs`
- Triggers: `cargo run --bin report`
- Responsibilities:
  1. Generate synthetic market data across parameter grids (T, skew slope)
  2. Run calibration on each scenario
  3. Compute fit quality metrics (max error, RMSE in implied vol)
  4. Generate SVG plots for each fit
  5. Create markdown report with constraint saturation analysis
  6. Write to `documents/fit_quality_report.md` and `documents/plots/`

**Test Entry (`tests/steep_skew.rs`):**
- Location: `tests/steep_skew.rs`
- Triggers: `cargo test steep_skew`
- Responsibilities: Stress-test calibration with near-zero expiry and steep skew
- Assertions: Convergence, maximum IV error bounds, parameter recovery

**Benchmark Entry (`benches/calibration.rs`):**
- Location: `benches/calibration.rs`
- Triggers: `cargo bench`
- Responsibilities: Measure performance of core operations
- Benchmarks: 20-point calibration, theta solving, total variance batch computation

## Error Handling

**Strategy:** Option-based (no exceptions in Rust)

**Patterns:**
- `solve_theta()` returns `Option<f64>` — None if Brent's method fails or bracket has no sign change
- `calibrate()` returns `Option<CalibrationResult>` — None if any inner solve_theta fails
- Brent's method returns `BrentResult` with `converged: bool` flag (checked by caller)
- Nelder-Mead always returns `NelderMeadResult` with `converged: bool` (no failure mode)
- No-arbitrage violations: Objective function penalized with 1e10 (not error, just very high cost)
- Invalid parameter ranges: Clamped by `project()` function in Nelder-Mead

## Cross-Cutting Concerns

**Logging:** No explicit logging framework; information flows through:
- Return values (Option, Result-like patterns)
- Print statements in binary (`src/bin/report.rs`)
- Test output for diagnostics

**Validation:**
- No-arbitrage constraint: Checked inline in objective function during calibration
- Parameter bounds: η ∈ [1e-6, 2], γ ∈ [1e-6, 1), ρ ∈ [-0.999, 0.999]
- Numerical stability: Explicit tolerance values (1e-14, 1e-12) in Brent and Nelder-Mead configs

**Implicit Constraint Elimination:**
- ATM consistency constraint (θ = θ* / (1 + ρ·φ(θ)·k*)) solved implicitly inside objective
- Reduces 4D search to 3D (or 2D grid + 3D polish)
- Guarantees constraint satisfaction in every candidate parameter set

---

*Architecture analysis: 2026-03-05*
