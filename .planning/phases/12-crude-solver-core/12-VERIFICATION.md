---
phase: 12-crude-solver-core
status: passed
verified: 2026-03-07
---

# Phase 12 Verification: Crude Solver Core

## Goal
A single volatility slice can be calibrated by directly optimizing all four SSVI parameters (theta, eta, gamma, rho) via bounded Nelder-Mead.

## Success Criteria

### 1. CrudeCalibConfig struct exists with theta bounds, 4D Nelder-Mead tolerances, lambda, and k_penalty grid settings, and has a Default impl
**Status: PASSED**
- `pub struct CrudeCalibConfig` at `src/crude_solver.rs:16`
- Fields: theta_lower/upper, eta_lower/upper, gamma_lower/upper, rho_lower/upper, nelder_mead (NelderMeadConfig), lambda, k_penalty_lo/hi/n
- `impl Default` at line 54 with sensible values (theta: 1e-6..1.0, lambda: 1.0, k_penalty_n: 50, max_iter: 5000)

### 2. Calling crude_solver::calibrate_slice with market data returns a CalibrationResult with (theta, eta, gamma, rho) that minimizes SSE on total variance
**Status: PASSED**
- `pub fn calibrate_slice` at `src/crude_solver.rs:149`
- Returns `CrudeCalibResult` with theta, eta, gamma, rho, sse, converged, iterations
- test_calibrate_slice_synthetic_flat: SSE < 1e-10 on exact data
- test_calibrate_slice_synthetic_skewed: SSE < 1e-8 on exact skewed data
- Multi-start optimization (12 starting points) ensures robust convergence

### 3. The objective function penalizes no-arbitrage butterfly violations, producing fits that satisfy the butterfly constraint on the fitted k-grid
**Status: PASSED**
- `butterfly_density()` computes g(k) via central finite differences (h=1e-5)
- `butterfly_penalty()` sums max(0, -g(k))^2 across penalty grid
- Objective = SSE + lambda * butterfly_penalty (line 192)
- test_calibrate_slice_butterfly_satisfied: g(k) >= -1e-6 for all k in [-1, 1]

### 4. All existing tests (111) continue to pass with no regressions
**Status: PASSED**
- Total tests: 115 (111 existing + 4 new)
- Failures: 0

## Requirements Traceability

| Requirement | Status | Evidence |
|-------------|--------|----------|
| CRUD-01 | Covered | CrudeCalibConfig struct with all specified fields and Default impl |
| CRUD-02 | Covered | calibrate_slice with 4D Nelder-Mead, SSE objective, butterfly penalty |

## Must-Haves Verification

| Must-Have | Status |
|-----------|--------|
| CrudeCalibConfig with theta bounds, NelderMeadConfig, lambda, k_penalty settings | PASSED |
| CrudeCalibConfig has Default impl | PASSED |
| calibrate_slice accepts (k_slice, w_market) and returns result with (theta, eta, gamma, rho) | PASSED |
| 4D Nelder-Mead optimizes all four parameters directly | PASSED |
| Objective includes butterfly penalty on k_penalty grid | PASSED |
| Fits satisfy butterfly constraint | PASSED |
| All 111 existing tests pass | PASSED (115 total, 0 failures) |

## Conclusion

All 4 success criteria passed. All 2 requirements covered. Phase 12 goal achieved.
