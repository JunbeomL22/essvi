---
phase: 16-sequential-surface-calibration
status: passed
verified: 2026-03-08
verifier: automated
---

# Phase 16: Sequential Surface Calibration -- Verification

## Phase Goal
Multiple volatility slices across expiries can be calibrated sequentially with monotonic theta enforcement via calendar spread penalty

## Success Criteria

### 1. calibrate_surface sorts slices by expiry T and calibrates from shortest to longest
**Status:** PASSED
**Evidence:**
- `src/direct_solver.rs:216-222`: Indices sorted by `slices[a].t.partial_cmp(&slices[b].t)`
- `src/direct_solver.rs:227-231`: Iterates in sorted order, calling `calibrate_slice_with_prev` per slice
- `tests/direct_solver.rs::test_calibrate_surface_monotonic_theta`: Passes 4 slices in scrambled order (T=0.5, 0.1, 1.0, 0.25), verifies results match T-sorted expected thetas (0.02, 0.04, 0.08, 0.15)

### 2. For each slice after the first, the objective includes a penalty term that discourages theta values below the previous slice's fitted theta
**Status:** PASSED
**Evidence:**
- `src/direct_solver.rs:225`: `prev_theta` initialized to `None`
- `src/direct_solver.rs:229-230`: Each slice calibrated with `prev_theta`, then `prev_theta = Some(result.theta)`
- `src/direct_solver.rs` (calibrate_slice_with_prev): Penalty `lambda_calendar * max(0, prev - theta)^2` added to objective when `prev_theta` is `Some`
- `tests/direct_solver.rs::test_calibrate_surface_calendar_penalty_effect`: lambda=0 recovers non-monotonic thetas; lambda=100 significantly reduces monotonicity gap

### 3. The resulting theta sequence across slices is monotonically non-decreasing (or nearly so, bounded by penalty strength)
**Status:** PASSED
**Evidence:**
- `tests/direct_solver.rs::test_calibrate_surface_monotonic_theta`: Asserts `theta[i+1] >= theta[i] - 1e-6` for all consecutive pairs -- PASSES
- `tests/direct_solver.rs::test_calibrate_surface_calendar_penalty_effect`: Demonstrates penalty reduces gap by >50% even when data strongly opposes monotonicity

## Requirement Coverage

| Requirement | Status | Evidence |
|-------------|--------|----------|
| CAL-01 | PASSED | calibrate_surface sorts by T, calibrates sequentially, includes calendar spread penalty on theta monotonicity |

## Test Results

- New tests: 5
- Total tests: 132
- Failures: 0
- Regressions: 0

## must_haves Verification

| must_have | Status |
|-----------|--------|
| calibrate_surface accepts &[SliceInput] and &DirectCalibConfig, returns Vec<DirectCalibResult> | PASSED |
| Slices sorted by T ascending before calibration | PASSED |
| First slice calibrated with no calendar penalty (prev_theta=None) | PASSED |
| Subsequent slices include lambda_calendar * max(0, prev_theta - theta)^2 penalty | PASSED |
| Theta monotonicity enforced when lambda_calendar sufficiently large | PASSED |
| All 127 existing tests pass with no regressions | PASSED (132 total) |

## Score

6/6 must_haves verified. All success criteria met.
