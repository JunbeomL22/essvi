---
phase: 15-direct-solver-core
status: passed
verified: 2026-03-08
verifier: automated
---

# Phase 15: Direct Solver Core — Verification

## Phase Goal

A single volatility slice can be calibrated via 4D bounded Nelder-Mead using only the algebraic no-arb condition eta*(1+|rho|) <= 2 as hard barrier, with multi-start search and post-hoc butterfly validation available.

## Success Criteria Verification

### 1. DirectCalibConfig struct exists with parameter bounds, multi-start grid settings, Nelder-Mead tolerances, and a Default impl

**Status: PASSED**

- `DirectCalibConfig` defined at `src/direct_solver.rs:20` with theta/eta/gamma/rho bounds, `NelderMeadConfig`, `rho_starts: Vec<f64>`, `eta_starts: Vec<f64>`, `lambda_calendar: f64`
- `Default` impl at line 59 with sensible defaults (max_iter: 5000, 4 rho starts, 3 eta starts)
- Verified by `test_direct_calib_config_default`

### 2. Calling direct_solver::calibrate_slice with market data returns a CalibrationResult where the optimizer enforces eta*(1+|rho|) <= 2 as a hard barrier and uses no butterfly density term in the objective

**Status: PASSED**

- `calibrate_slice` at `src/direct_solver.rs:192` calls `calibrate_slice_with_prev` with `None`
- Hard barrier at line 221: `if !ssvi::no_arbitrage_satisfied(eta, rho) { return 1e10; }`
- Objective is pure SSE (lines 224-230): `sum((w_model - w_market)^2)`, no butterfly penalty
- No `butterfly_penalty` or `lambda * butterfly` anywhere in objective function
- Verified by `test_no_arb_barrier_enforced` and `test_no_butterfly_in_objective`

### 3. Multi-start sweep over initial (rho, eta) values explores the parameter space to avoid local minima, and the best result (lowest SSE) is returned

**Status: PASSED**

- Multi-start loop at lines 277-289: iterates over `config.rho_starts` x `config.eta_starts`
- Tracks `best_f` and `best_res`, returns lowest-SSE result
- Default grid: 4 rho x 3 eta = 12 starting points
- Verified by `test_multi_start_finds_global_minimum` (SSE < 1e-6 near no-arb boundary)

### 4. validate_butterfly function accepts fitted SSVI params and returns per-point g(k) results indicating whether butterfly arbitrage is absent

**Status: PASSED**

- `validate_butterfly` at `src/direct_solver.rs:158` accepts `(theta, eta, gamma, rho, k_grid: &[f64])`
- Returns `Vec<ButterflyResult>` with `k`, `g` (density), `valid` (g >= 0)
- Uses `butterfly_density` helper with central finite differences (h=1e-5)
- Verified by `test_validate_butterfly` and `test_validate_butterfly_returns_per_point_detail`

### 5. All existing tests continue to pass with no regressions

**Status: PASSED**

- Total tests: 127 (119 existing + 8 new)
- All pass, 0 failures
- `cargo test` exits cleanly

## Requirement Traceability

| Requirement | Description | Status |
|-------------|-------------|--------|
| SOLV-01 | DirectCalibConfig struct + Default impl | PASSED |
| SOLV-02 | calibrate_slice with algebraic barrier, no butterfly | PASSED |
| SOLV-03 | Multi-start sweep over (rho, eta) | PASSED |
| VAL-01 | validate_butterfly with per-point g(k) | PASSED |

## must_haves Verification

| Truth | Verified |
|-------|----------|
| DirectCalibConfig with bounds, multi-start, NM config, Default | YES |
| calibrate_slice returns DirectCalibResult | YES |
| Objective uses only SSE (no butterfly density) | YES |
| eta*(1+\|rho\|) <= 2 hard barrier | YES |
| Multi-start returns best SSE | YES |
| validate_butterfly returns per-point g(k) | YES |
| 119 existing tests pass | YES (127 total) |

## Artifacts Verified

| Path | Exists | Contains |
|------|--------|----------|
| src/direct_solver.rs | YES | calibrate_slice, validate_butterfly, DirectCalibConfig |
| src/lib.rs | YES | pub mod direct_solver |
| tests/direct_solver.rs | YES | 8 tests including calibrate_slice |

## Result

**PASSED** -- All 5 success criteria met, all 4 requirements covered, all must_haves verified.
