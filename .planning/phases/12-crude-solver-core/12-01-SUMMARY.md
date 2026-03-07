---
phase: 12-crude-solver-core
plan: 01
status: complete
started: 2026-03-07
completed: 2026-03-07
---

# Plan 12-01 Summary: Crude Solver Core

## What was built

A new `crude_solver` module that calibrates SSVI parameters (theta, eta, gamma, rho) for a single volatility slice by directly optimizing all four parameters via bounded 4D Nelder-Mead. Unlike the existing `calibration::calibrate` which solves theta implicitly and sweeps rho on a grid, this approach treats all parameters as free variables with:

- **CrudeCalibConfig**: Configuration struct with theta bounds, eta/gamma/rho bounds, 4D Nelder-Mead settings (5000 max iterations), butterfly penalty weight (lambda), and penalty evaluation grid settings.
- **calibrate_slice**: Public calibration function with multi-start optimization (4 rho x 3 eta = 12 starting points) to avoid local minima.
- **Butterfly penalty**: Numerical evaluation of the SSVI butterfly density g(k) via central finite differences, penalizing violations in the objective.
- **Hard barrier**: The coupled no-arb condition eta*(1+|rho|) <= 2 is enforced by returning 1e10.

## Key decisions

- **Multi-start over single-start**: The 4D landscape has ridges from the (eta, gamma) degeneracy; 12 starting points across rho and eta ensure robust convergence.
- **Phi-based test assertions**: Since eta and gamma are inherently degenerate in SSVI (different pairs produce the same phi value), tests verify phi match and total variance curve match rather than individual eta/gamma recovery.
- **5000 max iterations**: The 4D search space requires more iterations than the 2D/3D default of 1000.

## Self-Check: PASSED

- [x] CrudeCalibConfig with Default impl
- [x] CrudeCalibResult struct
- [x] calibrate_slice with 4D Nelder-Mead
- [x] Butterfly penalty in objective
- [x] No-arb hard barrier
- [x] 4 integration tests pass
- [x] 115 total tests (111 + 4), 0 regressions

## Key files

### Created
- `src/crude_solver.rs` -- CrudeCalibConfig, CrudeCalibResult, calibrate_slice, butterfly helpers
- `tests/crude_solver.rs` -- 4 integration tests

### Modified
- `src/lib.rs` -- Added `pub mod crude_solver`

## Deviations

None. All plan tasks completed as specified.
