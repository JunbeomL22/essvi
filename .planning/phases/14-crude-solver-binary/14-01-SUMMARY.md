---
phase: 14-crude-solver-binary
plan: 01
subsystem: binary
tags: [ssvi, crude-solver, cboe, svg, binary]

requires:
  - phase: 12-crude-solver-core
    provides: "calibrate_slice, CrudeCalibConfig, CrudeCalibResult, butterfly penalty"
  - phase: 13-sequential-calibration
    provides: "calibrate_surface, SliceInput, lambda_calendar"
provides:
  - "fit_crude binary for end-to-end crude solver calibration on CBOE data"
  - "Per-slice SVG fit plots for visual inspection"
affects: []

tech-stack:
  added: []
  patterns: ["CLI binary wiring existing library modules to real data"]

key-files:
  created: [src/bin/fit_crude.rs]
  modified: []

key-decisions:
  - "Use total_variance column from CSV directly (crude solver works in w-space, avoids IV->w round-trip)"
  - "Reuse fit_common::plot_fit and FitResult for SVG rendering (no code duplication)"
  - "3-zone IV rule for plot display: p_iv (k < -0.1), mean (ATM), c_iv (k > 0.1)"
  - "Output directory named <ticker>_<date>_crude to distinguish from fit_cboe output"

patterns-established:
  - "Binary-as-integration-test pattern: run on real data to validate library code end-to-end"

requirements-completed: [BIN-01, BIN-02]

duration: 3 min
completed: 2026-03-07
---

# Plan 14-01 Summary: Crude Solver Binary

**Binary that runs crude_solver::calibrate_surface on CBOE SPX/NDX data with SVG fit plots**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-07
- **Completed:** 2026-03-07
- **Tasks:** 2
- **Files created:** 1

## Accomplishments
- `fit_crude` binary parses CBOE CSV data and calibrates all expiry slices via `crude_solver::calibrate_surface`
- Produces per-slice SVG plots showing SSVI model fit vs market implied volatility
- Prints formatted summary table with DTE, T, theta, eta, gamma, rho, SSE, converged
- Successfully tested on SPX (45 slices, 43 converged) and NDX (24 slices, 23 converged)
- Theta values are generally monotonically non-decreasing (calendar penalty working)

## Files Created
- `src/bin/fit_crude.rs` -- Binary with parse_cboe, fit_surface, report functions

## Decisions Made
- Used total_variance column from CSV directly instead of converting from IV (crude solver optimizes in total variance space)
- Reused fit_common::plot_fit for SVG rendering to avoid duplicating the plotters code
- Applied same 3-zone IV rule as fit_cboe.rs for consistent IV extraction

## Self-Check: PASSED

- [x] src/bin/fit_crude.rs exists and compiles cleanly (no warnings)
- [x] Binary uses crude_solver::calibrate_surface (NOT calibration::calibrate)
- [x] Binary parses CBOE CSV (log_moneyness, total_variance columns)
- [x] Per-slice SVG plots generated in documents/data-plots/<ticker>_<date>_crude/
- [x] Stdout summary table with all required columns
- [x] Handles SPX (45 slices) and NDX (24 slices) via CLI argument
- [x] All 119 existing tests pass, 0 failures
- [x] No panics on either dataset

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Cleanup] Removed unused w_market field from FitOutput**
- **Found during:** Task 1 (compilation)
- **Issue:** FitOutput had w_market field that was stored but never read (dead_code warning)
- **Fix:** Removed the field since the report function computes w_fit from fitted parameters, doesn't need w_market
- **Files modified:** src/bin/fit_crude.rs
- **Verification:** Clean build with no warnings

---

**Total deviations:** 1 auto-fixed (1 cleanup)
**Impact on plan:** Trivial field removal. No scope change.

## Issues Encountered
None.

## Milestone Readiness
- Phase 14 is the final phase of v1.3 Crude Solver
- All 3 phases (12, 13, 14) complete
- All v1.3 requirements covered: CRUD-01, CRUD-02, CRUD-03, BIN-01, BIN-02

---
*Phase: 14-crude-solver-binary*
*Completed: 2026-03-07*
