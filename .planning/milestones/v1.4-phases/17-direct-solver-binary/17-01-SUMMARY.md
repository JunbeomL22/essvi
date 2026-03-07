---
phase: 17-direct-solver-binary
plan: 01
subsystem: binary
tags: [ssvi, nelder-mead, direct-solver, svg, cboe, calibration]

requires:
  - phase: 15-direct-solver-core
    provides: "DirectCalibConfig, calibrate_slice, calibrate_surface, validate_butterfly"
  - phase: 16-sequential-surface-calibration
    provides: "calibrate_surface with calendar spread penalty on theta monotonicity"
provides:
  - "fit_direct binary for end-to-end direct solver calibration on CBOE data"
  - "Per-slice SVG fit plots (model vs market implied volatility)"
  - "Stdout fit summary table (DTE, T, theta, eta, gamma, rho, SSE, converged)"
affects: []

tech-stack:
  added: []
  patterns: ["Binary mirrors fit_crude structure (parse/fit/report) using direct_solver instead of crude_solver"]

key-files:
  created: ["src/bin/fit_direct.rs"]
  modified: []

key-decisions:
  - "Reused identical CSV parsing logic from fit_crude.rs (3-zone IV rule, log_moneyness focus range [-1,1], min 5 points per slice)"
  - "Direct solver uses pure SSE objective with algebraic no-arb barrier (no butterfly penalty in objective)"
  - "SVG output directory uses _direct suffix to distinguish from _crude plots"

patterns-established:
  - "Binary pattern: parse_cboe -> fit_surface -> report, reusable for future solver variants"

requirements-completed: [BIN-01, BIN-02]

duration: 3min
completed: 2026-03-08
---

# Phase 17: Direct Solver Binary Summary

**fit_direct binary calibrates 45 SPX and 24 NDX slices via direct_solver with algebraic no-arb barrier, multi-start sweep, SVG plots, and stdout summary**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-08
- **Completed:** 2026-03-08
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Created fit_direct binary that parses CBOE CSV data and calibrates all expiry slices via direct_solver::calibrate_surface
- All 45 SPX slices and all 24 NDX slices converge successfully with no panics
- Per-slice SVG fit plots generated (45 for SPX, 24 for NDX) showing model vs market implied volatility
- Theta values monotonically non-decreasing across expiries (calendar spread penalty working)
- All 132 existing tests pass with zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Create fit_direct binary** - `2c11ffc` (feat)
2. **Task 2: Run on SPX/NDX data** - validation only, no code changes

## Files Created/Modified
- `src/bin/fit_direct.rs` - Direct solver binary: CSV parse, calibrate via direct_solver::calibrate_surface, SVG plots, stdout summary

## Decisions Made
None - followed plan as specified

## Deviations from Plan
None - plan executed exactly as written

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 17 is the final phase of v1.4 Direct Solver milestone
- All v1.4 requirements (SOLV-01, SOLV-02, SOLV-03, VAL-01, CAL-01, BIN-01, BIN-02) are addressed across phases 15-17
- Milestone v1.4 is ready for completion

---
*Phase: 17-direct-solver-binary*
*Completed: 2026-03-08*
