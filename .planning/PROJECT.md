# essvi — Extended SVI Calibration Library

## What This Is

A pure Rust library for calibrating implied volatility surfaces using SVI-family parameterizations (SSVI and eSSVI). Provides derivative-free optimization (bounded Nelder-Mead), implicit theta resolution (Brent root-finding), and no-arbitrage enforcement. Targets quantitative finance practitioners who need fast, correct vol surface fitting with zero external dependencies.

## Core Value

Accurate, arbitrage-free implied volatility surface calibration that handles real-market skew profiles including steep put-side skew at all expiries.

## Requirements

### Validated

- SSVI model (phi function, total variance formula, no-arb check)
- Bounded Nelder-Mead optimizer (2D/3D with projection)
- Brent root finder for implicit theta solving
- SSVI calibration pipeline (rho-grid sweep + 2D NM + 3D polish)
- Benchmark report generator with SVG plots
- Inline unit tests and integration stress tests

### Active

- [x] CalibrationConfig struct (bounds, grid steps, tolerances, lambda) with Default impl
- [x] Module restructuring (solver/, model/ submodules)
- [x] Impl blocks on domain structs (CalibrationResult, CalibrationInput, etc.)
- [x] Proper error types (Result<T, CalibError> replacing Option<T>)
- [ ] Deduplicate binary code (shared SliceData, make_slice, FitResult, plot_fit)
- [ ] Move all inline #[cfg(test)] blocks to tests/ directory

### Out of Scope

- eSSVI model implementation — deferred to next milestone
- Surface-level calibration improvements — future milestone
- Real market data parsing — future milestone
- API ergonomics / crate publishing — future milestone

## Context

- The current codebase works correctly but has structural debt: hardcoded numeric constants scattered across calibration functions, duplicated code between binaries, flat module structure, and inline tests mixed with production code.
- All calibration bounds (eta: [1e-6, 2.0-1e-6], gamma: [1e-6, 1.0-1e-6], rho: [-0.999, 0.999]), grid parameters (n_rho=20, rho sweep -0.95 to 0.95), and penalty config (k_penalty: -0.5 to 0.5 step 0.05, lambda=100) are hardcoded.
- fit_real.rs and fit_real_surface.rs share ~150 lines of identical code (SliceData, make_slice, build_market_slices, FitResult, plot_fit).
- Error handling uses Option<T> throughout the library — no way to distinguish failure modes.

## Constraints

- **Tech stack**: Pure Rust, zero external dependencies for core logic (plotters allowed for reporting only)
- **Backwards compat**: Public API changes must not break existing binary targets (update them in the same milestone)
- **Testing**: All existing tests must pass after restructuring (move, don't delete)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Bounded Nelder-Mead over L-BFGS-B | Only 3 vars, avoids external C deps | Good |
| Eliminate equality constraint via implicit theta | Simpler solver, better convergence | Good |
| Single CalibrationConfig struct | Keeps API surface small, one place for all tuning knobs | — Pending |
| Result<T, CalibError> over Option<T> | Enables callers to distinguish and handle failure modes | Good |
| Reorganize into solver/ and model/ submodules | Clearer separation of concerns as library grows | Good |

## Current Milestone: v1.0 Idiomatic Restructuring

**Goal:** Make the library structurally clean, configurable, and idiomatic Rust — removing hardcoded values, adding proper error types, restructuring modules, and separating tests.

**Target features:**
- CalibrationConfig struct with Default impl
- Module hierarchy (solver/, model/)
- Impl blocks on all domain structs
- Proper error types with CalibError
- Deduplicated binary code
- External test directory mirroring src/ structure

---
*Last updated: 2026-03-07 after milestone v1.0 initialization*
