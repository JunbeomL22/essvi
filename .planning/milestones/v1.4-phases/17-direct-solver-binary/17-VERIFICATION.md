---
phase: 17-direct-solver-binary
status: passed
verified: 2026-03-08
requirements_verified: [BIN-01, BIN-02]
---

# Phase 17: Direct Solver Binary - Verification

## Phase Goal
User can run a single command to calibrate real SPX/NDX option chains with direct_solver and inspect fit quality via SVG plots

## Success Criteria Verification

### 1. Running `cargo run --bin fit_direct` parses CSV market data from data/cboe/ and calibrates all slices via direct_solver
**Status:** PASSED
- Binary exists at `src/bin/fit_direct.rs` and compiles
- Uses `direct_solver::calibrate_surface` (3 references, 0 crude_solver references)
- Successfully parsed and calibrated 45 SPX slices and 24 NDX slices

### 2. The binary outputs per-slice SVG plots showing model total variance vs market total variance across log-moneyness
**Status:** PASSED
- SPX: 45 SVG plots in `documents/data-plots/spx_2026-03-07_direct/`
- NDX: 24 SVG plots in `documents/data-plots/ndx_2026-03-07_direct/`
- All SVG files are well-formed (19-44KB each)

### 3. The binary prints per-slice fit summary (T, theta, eta, gamma, rho, SSE) to stdout
**Status:** PASSED
- Summary table printed with columns: DTE, T, theta, eta, gamma, rho, SSE, converged
- All slices show converged=yes for both SPX and NDX

### 4. The binary handles both SPX and NDX data without code changes (parameterized by path or argument)
**Status:** PASSED
- Ticker is a CLI argument: `cargo run --bin fit_direct -- <ticker> <date>`
- Both SPX and NDX run successfully with the same binary

## Requirement Coverage

| Requirement | Description | Status |
|-------------|-------------|--------|
| BIN-01 | fit_direct binary parses CBOE CSV data and calibrates all slices via direct_solver | PASSED |
| BIN-02 | Binary outputs per-slice SVG plots and prints fit summary to stdout | PASSED |

## must_haves Verification

| Truth | Status |
|-------|--------|
| Running `cargo run --bin fit_direct -- <ticker> <date>` parses CSV and calibrates via direct_solver::calibrate_surface | PASSED |
| Binary outputs per-slice SVG plots to documents/data-plots/<ticker>_<date>_direct/ | PASSED |
| Binary prints per-slice fit summary table with DTE, T, theta, eta, gamma, rho, SSE, converged | PASSED |
| Binary handles both SPX and NDX without code changes | PASSED |
| CSV parser uses log_moneyness and total_variance columns directly | PASSED |
| Slices with fewer than 5 data points are skipped | PASSED |
| All 132 existing tests continue to pass | PASSED |

## Regression Check
- All 132 tests pass with 0 failures
- No library code modified (binary-only change)

## Overall Result

**PASSED** - All 4 success criteria met, all 2 requirements verified, all 7 must_haves confirmed.
