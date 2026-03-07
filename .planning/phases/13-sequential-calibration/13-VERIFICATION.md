---
phase: 13-sequential-calibration
status: passed
verified: 2026-03-07
---

# Phase 13 Verification: Sequential Calibration

## Goal
Multiple volatility slices across expiries can be calibrated sequentially with monotonic theta enforcement via calendar spread penalty.

## Success Criteria

### 1. Slices are sorted by expiry T and calibrated from shortest to longest
**Status: PASSED**
- `calibrate_surface` at `src/crude_solver.rs:182` sorts indices by T ascending via `indices.sort_by`
- `test_calibrate_surface_monotonic_theta` passes slices in scrambled order (T=0.5, 0.1, 1.0, 0.25) and verifies results are returned in sorted order with correct theta values

### 2. For each slice after the first, the objective function includes a lambda-weighted penalty term that penalizes theta values below the previous slice's fitted theta
**Status: PASSED**
- `calibrate_slice_with_prev` at line 210 accepts `prev_theta: Option<f64>`
- When `prev_theta` is `Some(prev)`, objective adds `lambda_cal * shortfall * shortfall` where `shortfall = (prev - theta).max(0.0)` (line 265-267)
- `test_calibrate_surface_calendar_penalty_effect` confirms: with lambda_calendar=0 the non-monotonic data is recovered as-is; with lambda_calendar=1000 the monotonicity gap shrinks by >50%

### 3. The resulting theta sequence across slices is monotonically non-decreasing (or nearly so, with violations bounded by lambda strength)
**Status: PASSED**
- `test_calibrate_surface_monotonic_theta` asserts `theta[i+1] >= theta[i] - 1e-6` for all consecutive pairs
- Default lambda_calendar=10.0 provides soft enforcement; stronger values provide tighter enforcement
- `test_calibrate_surface_calendar_penalty_effect` demonstrates the tradeoff between penalty strength and fit quality

## Requirements Traceability

| Requirement | Status | Evidence |
|-------------|--------|----------|
| CRUD-03 | Covered | calibrate_surface with sequential calibration and calendar spread penalty |

## Must-Haves Verification

| Must-Have | Status |
|-----------|--------|
| calibrate_surface accepts SliceInput slice and returns Vec<CrudeCalibResult> | PASSED |
| Slices sorted by T before calibration regardless of input order | PASSED |
| First slice calibrated identically to calibrate_slice (no calendar penalty) | PASSED |
| Subsequent slices include lambda_calendar * max(0, prev_theta - theta)^2 penalty | PASSED |
| Theta sequence monotonically non-decreasing with sufficient lambda | PASSED |
| CrudeCalibConfig has lambda_calendar field with default 10.0 | PASSED |
| All 115 existing tests pass (now 119 total, 0 failures) | PASSED |

## Conclusion

All 3 success criteria passed. CRUD-03 requirement covered. Phase 13 goal achieved.
