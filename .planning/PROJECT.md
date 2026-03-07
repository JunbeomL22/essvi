# essvi — Extended SVI Calibration Library

## What This Is

A pure Rust library for calibrating implied volatility surfaces using SVI-family parameterizations (SSVI and eSSVI). Provides Black-76 option pricing, machine-precision implied volatility recovery (Let's Be Rational), derivative-free optimization (bounded Nelder-Mead), implicit theta resolution (Brent root-finding), configurable calibration parameters, proper error types, and no-arbitrage enforcement. Organized into `math/`, `pricing/`, `solver/`, and `model/` submodules with all tests in `tests/`.

## Core Value

Accurate, arbitrage-free implied volatility surface calibration that handles real-market skew profiles including steep put-side skew at all expiries.

## Requirements

### Validated

- ✓ SSVI model (phi function, total variance formula, no-arb check) — v1.0
- ✓ Bounded Nelder-Mead optimizer (2D/3D with projection) — v1.0
- ✓ Brent root finder for implicit theta solving — v1.0
- ✓ SSVI calibration pipeline (rho-grid sweep + 2D NM + 3D polish) — v1.0
- ✓ Benchmark report generator with SVG plots — v1.0
- ✓ CalibrationConfig struct (bounds, grid steps, tolerances) with Default impl — v1.0
- ✓ Module restructuring (solver/, model/ submodules) — v1.0
- ✓ Impl blocks on domain structs (CalibrationResult methods) — v1.0
- ✓ Proper error types (Result<T, CalibError> replacing Option<T>) — v1.0
- ✓ Deduplicated binary code (shared SliceData, make_slice, FitResult, plot_fit) — v1.0
- ✓ External test directory mirroring src/ structure — v1.0
- ✓ Black-76 pricing model (undiscounted price/delta, greeks, gamma, vega, theta) — v1.1
- ✓ Let's Be Rational implied volatility solver (Jaeckel's machine-precision algorithm) — v1.1
- ✓ Supporting math: Cody's erf/erfc/erfcx, normal distribution (standard + high-precision) — v1.1
- ✓ Rational cubic interpolation utilities — v1.1
- ✓ PricingError type and numerical constants — v1.1

### Active

- [ ] Real option price data for European-style index options (Euro Stoxx 50 / Nikkei 225, fallback SPX)
- [ ] `data/` directory with organized market data files

## Current Milestone: v1.2 Market Data Collection

**Goal:** Collect and store real European-style index option price data for testing and validation.

**Target features:**
- `data/` directory for market data storage
- Real option price data (Euro Stoxx 50 or Nikkei 225 preferred, SPX fallback)
- At least 1 day of data, manually downloaded from reliable public sources

### Out of Scope

- eSSVI model implementation — deferred to future milestone
- Surface-level calibration improvements — future milestone
- Real market data parsing — deferred (v1.2 is collection only, parsing in future milestone)
- API ergonomics / crate publishing — future milestone
- Async/parallel calibration — not needed for current use case
- Integration with existing calibration pipeline — will be wired in a later milestone when input is option prices instead of IVs

## Context

Shipped v1.1 with 4,663 LOC Rust. Tech stack: pure Rust, plotters for reporting.
Module structure: `src/math/{erf,normal,normal_hp,constants}.rs`, `src/pricing/{black76,error,lets_be_rational,rational_cubic}.rs`, `src/model/ssvi.rs`, `src/solver/{nelder_mead,brent}.rs`, `src/calibration.rs`, `src/fit_common.rs`.
111 tests (91 integration in `tests/`, 20 doc-tests), all passing.

## Constraints

- **Tech stack**: Pure Rust, zero external dependencies for core logic (plotters allowed for reporting only)
- **Backwards compat**: Public API changes must not break existing binary targets (update them in the same milestone)
- **Testing**: All existing tests must pass after restructuring (move, don't delete)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Bounded Nelder-Mead over L-BFGS-B | Only 3 vars, avoids external C deps | ✓ Good |
| Eliminate equality constraint via implicit theta | Simpler solver, better convergence | ✓ Good |
| Single CalibrationConfig struct | Keeps API surface small, one place for all tuning knobs | ✓ Good |
| Result<T, CalibError> over Option<T> | Enables callers to distinguish and handle failure modes | ✓ Good |
| Reorganize into solver/ and model/ submodules | Clearer separation of concerns as library grows | ✓ Good |
| CalibError with 4 variants (NonPositiveTheta, ZeroDerivative, ThetaDivergence, NonConvergence) | Covers all calibration failure modes without over-granularity | ✓ Good |
| k_penalty/lambda as direct params, not in CalibrationConfig | Surface-level concerns, not per-slice calibration knobs | ✓ Good |
| Shared binary code in src/fit_common.rs (library module) | Cargo auto-discovers src/bin/*.rs as binaries; library module is idiomatic | ✓ Good |
| FitResult superset struct with default 0 for calendar fields | Avoids Option wrapping, keeps struct simple | ✓ Good |
| fdlibm-based erf/erfc over custom polynomial | Battle-tested reference implementation, machine-precision | ✓ Good |
| Acklam + Halley for inverse CDF | Fast initial approximation with quadratic convergence refinement | ✓ Good |
| Asymptotic expansion for normal CDF tails | Avoids premature 0/1 clamping at extreme arguments | ✓ Good |
| Bisection + Halley for implied volatility | Robust initial bracket, machine-precision in 2 iterations | ✓ Good |
| i32 q (+1/-1) convention over OptionType enum | Matches Let's Be Rational API directly, avoids unnecessary abstraction | ✓ Good |
| PricingError with 3 variants | Covers all pricing failure modes (above max, below intrinsic, invalid input) | ✓ Good |

---
*Last updated: 2026-03-07 after v1.2 milestone start*
