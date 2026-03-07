# Requirements: essvi

**Defined:** 2026-03-08
**Core Value:** Accurate, arbitrage-free implied volatility surface calibration that handles real-market skew profiles including steep put-side skew at all expiries

## v1.4 Requirements

Requirements for v1.4 Direct Solver. Each maps to roadmap phases.

### Solver Core

- [ ] **SOLV-01**: direct_solver module exists with `DirectCalibConfig` struct and `Default` impl
- [ ] **SOLV-02**: `calibrate_slice` optimizes (theta, eta, gamma, rho) via 4D bounded Nelder-Mead with `eta*(1+|rho|) <= 2` as hard barrier only (no butterfly density in objective)
- [ ] **SOLV-03**: Multi-start sweep over initial (rho, eta) to avoid local minima

### Calibration

- [ ] **CAL-01**: `calibrate_surface` sorts slices by T and calibrates sequentially with calendar spread penalty on theta monotonicity

### Validation

- [ ] **VAL-01**: `validate_butterfly` function checks g(k) >= 0 on fitted params and returns per-point validation results

### Binary

- [ ] **BIN-01**: `fit_direct` binary parses CBOE CSV data and calibrates all slices via direct_solver
- [ ] **BIN-02**: Binary outputs per-slice SVG plots and prints fit summary (T, theta, eta, gamma, rho, SSE) to stdout

## v1.3 Requirements (Shipped)

- [x] **CRUD-01**: CrudeCalibConfig struct with theta bounds, 4D Nelder-Mead config, lambda, and k_penalty grid settings
- [x] **CRUD-02**: Single-slice calibration via 4D Nelder-Mead over (theta, eta, gamma, rho) with SSE on total variance + no-arb penalty
- [x] **CRUD-03**: Sequential multi-slice calibration — sort by T, calibrate shortest first, penalize theta monotonicity violations
- [x] **BIN-01**: Binary that parses CSV market data and runs crude_solver on SPX/NDX option chains
- [x] **BIN-02**: SVG fit plots per slice showing model vs market total variance

## Future Requirements

### Data Pipeline

- **DATA-01**: Dedicated CSV parser module for CBOE option chain data
- **DATA-02**: Put-call parity forward extraction from option prices
- **DATA-03**: Automated CalibrationInput construction from raw CSV

### Surface Calibration

- **SURF-01**: eSSVI model implementation
- **SURF-02**: Joint surface-level calibration across all slices

### Quality Comparison

- **COMP-01**: Compare direct_solver vs crude_solver fit quality on same data
- **COMP-02**: Benchmark runtime direct_solver vs crude_solver

## Out of Scope

| Feature | Reason |
|---------|--------|
| Modifying crude_solver.rs | Kept as reference; new module instead |
| Butterfly density in objective function | Algebraic condition eta*(1+\|rho\|) <= 2 is sufficient for SSVI phi |
| eSSVI model | Deferred to future milestone |
| Modifying existing calibration.rs | Existing implicit-theta solver stays as-is |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SOLV-01 | TBD | Pending |
| SOLV-02 | TBD | Pending |
| SOLV-03 | TBD | Pending |
| CAL-01 | TBD | Pending |
| VAL-01 | TBD | Pending |
| BIN-01 | TBD | Pending |
| BIN-02 | TBD | Pending |

**Coverage:**
- v1.4 requirements: 7 total
- Mapped to phases: 0
- Unmapped: 7

---
*Requirements defined: 2026-03-08*
*Last updated: 2026-03-08 after initial definition*
