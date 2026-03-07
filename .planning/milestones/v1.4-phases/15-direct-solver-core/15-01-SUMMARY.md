---
phase: 15-direct-solver-core
plan: "01"
status: completed
started: 2026-03-08
completed: 2026-03-08
---

# Plan 15-01: Direct Solver Core — Summary

## What Was Built

Direct SSVI solver module (`src/direct_solver.rs`) implementing single-slice calibration via 4D bounded Nelder-Mead with:

1. **DirectCalibConfig** — parameter bounds (theta, eta, gamma, rho), multi-start grid settings (4 rho starts x 3 eta starts = 12 initial points), Nelder-Mead tolerances (5000 max iterations), calendar spread penalty weight
2. **calibrate_slice** — pure SSE objective (no butterfly density penalty), algebraic no-arb barrier eta*(1+|rho|) <= 2 via ssvi::no_arbitrage_satisfied
3. **Multi-start sweep** — iterates over rho_starts x eta_starts grid, returns best result (lowest SSE)
4. **validate_butterfly** — public post-hoc checker returning per-point ButterflyResult with g(k) density and valid flag
5. **calibrate_slice_with_prev** — supports optional prev_theta for calendar spread penalty (Phase 16 prep)

## Key Design Decisions

- **No butterfly in objective**: Unlike crude_solver, the direct solver uses pure SSE. The algebraic condition eta*(1+|rho|) <= 2 is sufficient for SSVI phi. Butterfly validation is available post-hoc.
- **Multi-start as first-class**: Config stores rho_starts and eta_starts as Vec<f64>, making the grid user-configurable.
- **Parameter non-uniqueness**: Tests focus on SSE quality rather than exact parameter recovery, since (eta, gamma) have a trade-off in the SSVI phi function.

## Self-Check: PASSED

| Criterion | Status |
|-----------|--------|
| DirectCalibConfig with bounds, multi-start, NM config, Default | PASS |
| calibrate_slice with pure SSE + algebraic barrier | PASS |
| Multi-start sweep returns best SSE result | PASS |
| validate_butterfly with per-point g(k) results | PASS |
| All 127 tests pass (119 existing + 8 new) | PASS |

## key-files

### created
- src/direct_solver.rs
- tests/direct_solver.rs

### modified
- src/lib.rs

## Commits

1. `23847d8` — feat(15): add direct_solver module with DirectCalibConfig, calibrate_slice, and validate_butterfly
2. `99bf5a7` — test(15): add 8 integration tests for direct_solver module

## Deviations

None.

## Issues

None.
