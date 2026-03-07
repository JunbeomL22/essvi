---
phase: 14-crude-solver-binary
status: passed
verified: 2026-03-07
---

# Phase 14 Verification: Crude Solver Binary

## Goal
User can run a single command to calibrate real SPX/NDX option chains with crude_solver and inspect fit quality via SVG plots.

## Success Criteria

### 1. Running `cargo run --bin fit_crude` parses CSV market data from data/cboe/ and calibrates all slices
**Status: PASSED**
- `src/bin/fit_crude.rs` implements `parse_cboe` that reads CSV with columns log_moneyness, total_variance, c_iv, p_iv
- Groups by DTE, filters to [-1.0, 1.0] log-moneyness range, requires >= 5 points per slice
- Uses `crude_solver::calibrate_surface` for sequential calibration with calendar spread penalty
- SPX run: 45 slices parsed and calibrated (43/45 converged)
- NDX run: 24 slices parsed and calibrated (23/24 converged)

### 2. The binary outputs per-slice SVG plots showing model total variance vs market total variance across log-moneyness
**Status: PASSED**
- 45 SVG files generated in `documents/data-plots/spx_2026-03-07_crude/`
- 24 SVG files generated in `documents/data-plots/ndx_2026-03-07_crude/`
- All files > 19KB (well-formed SVG with market dots + SSVI fit curve)
- Uses `fit_common::plot_fit` for rendering (model vs market in IV space)

### 3. The binary prints per-slice fit summary (T, theta, eta, gamma, rho, SSE) to stdout
**Status: PASSED**
- Summary table printed with columns: DTE, T, theta, eta, gamma, rho, SSE, converged
- Per-slice detail lines printed during calibration

### 4. The binary handles both SPX and NDX data without code changes (parameterized by path or argument)
**Status: PASSED**
- Ticker is first CLI argument: `cargo run --bin fit_crude -- spx 2026-03-07`
- Same binary handles SPX (45 slices) and NDX (24 slices) without modification

## Requirements Traceability

| Requirement | Status | Evidence |
|-------------|--------|----------|
| BIN-01 | Covered | fit_crude binary parses CSV and runs crude_solver on SPX/NDX |
| BIN-02 | Covered | Per-slice SVG plots generated in documents/data-plots/ |

## Must-Haves Verification

| Must-Have | Status |
|-----------|--------|
| `cargo run --bin fit_crude -- <ticker> <date>` works | PASSED |
| SVG plots in documents/data-plots/<ticker>_<date>_crude/ | PASSED |
| Stdout summary table with DTE, T, theta, eta, gamma, rho, SSE, converged | PASSED |
| Handles both SPX and NDX via CLI argument | PASSED |
| CSV parser uses log_moneyness and total_variance columns directly | PASSED |
| Slices with < 5 points skipped | PASSED |
| All 119 existing tests pass | PASSED |

## Conclusion

All 4 success criteria passed. BIN-01 and BIN-02 requirements covered. Phase 14 goal achieved.
