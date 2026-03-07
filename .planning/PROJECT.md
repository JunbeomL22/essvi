# essvi — Extended SVI Calibration Library

## What This Is

A pure Rust library for calibrating implied volatility surfaces using SVI-family parameterizations (SSVI and eSSVI). Provides derivative-free optimization (bounded Nelder-Mead), implicit theta resolution (Brent root-finding), configurable calibration parameters, proper error types, and no-arbitrage enforcement. Organized into `solver/` and `model/` submodules with all tests in `tests/`.

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

### Active

(None — ready for next milestone)

### Out of Scope

- eSSVI model implementation — deferred to next milestone
- Surface-level calibration improvements — future milestone
- Real market data parsing — future milestone
- API ergonomics / crate publishing — future milestone
- Async/parallel calibration — not needed for current use case

## Context

Shipped v1.0 with 2,284 LOC Rust. Tech stack: pure Rust, plotters for reporting.
Module structure: `src/model/ssvi.rs`, `src/solver/{nelder_mead,brent}.rs`, `src/calibration.rs`, `src/fit_common.rs`.
14 tests (12 unit in `tests/`, 2 integration in `tests/steep_skew.rs`), all passing.

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

---
*Last updated: 2026-03-07 after v1.0 milestone*
