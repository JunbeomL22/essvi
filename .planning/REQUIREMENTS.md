# Requirements: essvi

**Defined:** 2026-03-07
**Core Value:** Accurate, arbitrage-free implied volatility surface calibration that handles real-market skew profiles

## v1.3 Requirements

Requirements for crude solver milestone. Each maps to roadmap phases.

### Solver Core

- [ ] **CRUD-01**: CrudeCalibConfig struct with theta bounds, 4D Nelder-Mead config, lambda, and k_penalty grid settings
- [ ] **CRUD-02**: Single-slice calibration via 4D Nelder-Mead over (theta, eta, gamma, rho) with SSE on total variance + no-arb penalty
- [ ] **CRUD-03**: Sequential multi-slice calibration — sort by T, calibrate shortest first, penalize theta monotonicity violations via lambda * calendar spread penalty

### Binary

- [ ] **BIN-01**: Binary that parses CSV market data and runs crude_solver on SPX/NDX option chains
- [ ] **BIN-02**: SVG fit plots per slice showing model vs market total variance

## Future Requirements

### Data Pipeline

- **DATA-01**: Dedicated CSV parser module for CBOE option chain data
- **DATA-02**: Put-call parity forward extraction from option prices
- **DATA-03**: Automated CalibrationInput construction from raw CSV

### Surface Calibration

- **SURF-01**: eSSVI model implementation
- **SURF-02**: Joint surface-level calibration across all slices

## Out of Scope

| Feature | Reason |
|---------|--------|
| eSSVI model | Deferred to future milestone |
| Modifying existing calibration.rs | Existing implicit-theta solver stays as-is |
| API publishing / crate ergonomics | Future milestone |
| CSV parser as separate module | Binary handles parsing inline for now |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CRUD-01 | -- | Pending |
| CRUD-02 | -- | Pending |
| CRUD-03 | -- | Pending |
| BIN-01 | -- | Pending |
| BIN-02 | -- | Pending |

**Coverage:**
- v1.3 requirements: 5 total
- Mapped to phases: 0
- Unmapped: 5

---
*Requirements defined: 2026-03-07*
*Last updated: 2026-03-07 after initial definition*
