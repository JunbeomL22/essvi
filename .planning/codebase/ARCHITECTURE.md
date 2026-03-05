# Architecture

**Analysis Date:** 2026-03-05

## Pattern Overview

**Overall:** Library crate (no binary entry point)

**Key Characteristics:**
- Pure Rust library for SSVI (Surface Stochastic Volatility Inspired) implied volatility surface calibration
- Zero external dependencies -- all numerical algorithms implemented from scratch
- Early-stage project: only a scaffold `lib.rs` exists; the design document (`documents/guideline.md`) describes the full intended architecture
- The project targets a bounded Nelder-Mead optimizer for calibrating SSVI model parameters (eta, gamma, rho)

## Intended Layers (from design document)

The guideline document (`documents/guideline.md`) lays out a multi-layer architecture for SSVI calibration. The codebase has not yet implemented these layers, but the design intent is:

**Optimization Layer:**
- Purpose: Bounded Nelder-Mead simplex optimizer (derivative-free)
- Planned location: `src/` (likely `src/optimizer.rs` or `src/nelder_mead.rs`)
- Contains: `NelderMeadConfig`, `NelderMeadResult`, `nelder_mead_bounded()` function, `project()` helper
- Depends on: Nothing (standalone numerical code)
- Used by: SSVI calibration layer

**SSVI Model Layer:**
- Purpose: SSVI total variance surface formula and phi function
- Planned location: `src/` (likely `src/ssvi.rs` or `src/model.rs`)
- Contains: `w(k, theta)` surface formula, `phi(theta)` power-law function
- Depends on: Nothing
- Used by: Calibration layer

**Calibration Layer:**
- Purpose: Orchestrates optimization by constructing the objective function
- Planned location: `src/` (likely `src/calibration.rs`)
- Contains: Objective function assembly, theta solver (Brent root finding or fixed-point iteration), loss computation
- Depends on: Optimization layer, SSVI model layer
- Used by: External consumers of the library

## Data Flow

**SSVI Calibration Flow (designed, not yet implemented):**

1. Caller provides market total variance data (`W`), ATM parameters (`theta_star`, `k_star`), and strike log-moneyness array (`k_array`)
2. Optimizer proposes candidate parameters `(eta, gamma, rho)` within bounds
3. For each candidate, theta is solved from the implicit equation `theta = g(theta; eta, gamma, rho)` via Brent root finding or fixed-point iteration
4. SSVI surface `w(k, theta)` is computed across all strikes
5. Squared error `||W - w_model||^2` is returned to the optimizer
6. Optimizer adjusts simplex and repeats until convergence
7. Calibrated `(eta, gamma, rho)` returned as `NelderMeadResult`

**State Management:**
- Stateless functional design -- optimizer takes a closure and returns a result struct
- No mutable global state; all state is local to the optimization loop (simplex vertices and function values)

## Key Abstractions

**NelderMeadConfig:**
- Purpose: Controls optimizer behavior (tolerances, iteration limit, simplex coefficients)
- Pattern: Struct with `Default` impl providing sensible defaults (tol_f=1e-12, tol_x=1e-12, max_iter=1000)

**NelderMeadResult:**
- Purpose: Encapsulates optimization output (best point, function value, iteration count, convergence flag)
- Pattern: Plain data struct with `Debug, Clone` derives

**nelder_mead_bounded():**
- Purpose: Core optimization function -- generic over any `Fn(&[f64]) -> f64` objective
- Pattern: Free function taking closure + initial guess + bounds + config, returning result

## Entry Points

**Library Entry Point:**
- Location: `src/lib.rs`
- Current state: Contains only a placeholder `add()` function and a single test
- Future role: Will re-export optimizer, model, and calibration modules

## Error Handling

**Strategy:** Not yet implemented. The design in `guideline.md` uses `unwrap()` on `partial_cmp` inside the optimizer, suggesting a panic-on-NaN approach for numerical failures.

**Expected Patterns:**
- Convergence reported via `NelderMeadResult.converged` boolean rather than `Result` type
- Bound violations handled by projection (clamping), not errors
- NaN/Inf in objective function values would cause panics in sort comparisons

## Cross-Cutting Concerns

**Logging:** Not present. No logging framework planned.
**Validation:** Bound enforcement via `project()` (clamping to `[lb, ub]`). No input validation on dimensions or bound ordering.
**Authentication:** Not applicable (library crate).

## Design Decisions (from guideline)

**Why Nelder-Mead over other solvers:**
- Only 3 optimization variables -- small-scale problem where derivative-free methods work well
- `argmin` crate lacks L-BFGS-B; bounded Nelder-Mead avoids external C dependencies (unlike `nlopt`)
- Approximately 170 lines of code for a complete implementation
- Objective function gradient would require implicit function theorem (computing d_theta/d_eta etc.), which is unnecessarily complex for 3 variables

**Why eliminate the equality constraint:**
- ATM consistency constraint makes theta implicitly determined by (eta, gamma, rho)
- Solving theta inside the objective function reduces the problem from "3 variables + equality constraint + bounds" to "3 variables + bounds only"
- Simpler solver requirements, better convergence stability

---

*Architecture analysis: 2026-03-05*
