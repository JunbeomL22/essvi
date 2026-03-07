---
phase: 16-sequential-surface-calibration
plan: 01
status: completed
started: 2026-03-08
completed: 2026-03-08
---

# Plan 16-01: Sequential Surface Calibration

## Result

Added `calibrate_surface` to `direct_solver.rs` for sequential multi-slice SSVI calibration with calendar spread penalty on theta monotonicity. 5 new integration tests validate sorting, monotonicity enforcement, penalty effect, edge cases, and regression safety.

## Tasks Completed

| # | Task | Status |
|---|------|--------|
| 1 | Implement calibrate_surface in direct_solver | Done |
| 2 | Add integration tests for sequential surface calibration | Done |

## Key Files

### Created
- (none -- extended existing files)

### Modified
- `src/direct_solver.rs` -- added `calibrate_surface` public function
- `tests/direct_solver.rs` -- added 5 new tests (13 total in file, 132 total project)

## Test Results

- Tests added: 5
- Tests passing: 132 (127 existing + 5 new)
- Failures: 0
- Regressions: 0

## Self-Check: PASSED

All must_haves verified:
- calibrate_surface accepts &[SliceInput] and &DirectCalibConfig, returns Vec<DirectCalibResult>
- Slices sorted by T ascending before calibration (verified with scrambled input)
- First slice calibrated with no calendar penalty (prev_theta=None)
- Subsequent slices include lambda_calendar * max(0, prev_theta - theta)^2 penalty
- Theta monotonicity enforced when lambda_calendar is sufficiently large
- All 127 pre-existing tests pass with no regressions

## Deviations

- Calendar penalty effect test uses relative gap reduction assertion rather than absolute monotonicity check, since the soft quadratic penalty with lambda=100 reduces but does not fully eliminate violations when data strongly opposes monotonicity (theta drops from 0.10 to 0.05). This correctly reflects the soft-penalty design.

## Commits

1. `59c464b` feat(16-01): add calibrate_surface for sequential multi-slice calibration
2. `6076df3` test(16-01): add integration tests for sequential surface calibration
