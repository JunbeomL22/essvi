# Requirements: essvi

**Defined:** 2026-03-07
**Core Value:** Accurate, arbitrage-free implied volatility surface calibration

## v1.1 Requirements

Requirements for Pricing Primitives milestone. Each maps to roadmap phases.

### Math Foundations

- [x] **MATH-01**: Library provides Cody's erf/erfc/erfcx with rational Chebyshev approximations
- [x] **MATH-02**: Library provides standard-precision normal distribution (PDF, CDF, inverse CDF)
- [x] **MATH-03**: Library provides high-precision normal distribution (PDF, CDF with asymptotic expansion, inverse CDF)
- [ ] **MATH-04**: Library provides rational cubic interpolation for initial guess refinement
- [x] **MATH-05**: Library provides numerical constants module (machine epsilon, algorithm thresholds)

### Black-76 Pricing

- [ ] **BLK-01**: User can compute undiscounted option price and delta for calls/puts
- [ ] **BLK-02**: User can compute individual greeks (gamma, vega, theta) and combined gamma_vega
- [ ] **BLK-03**: User can compute all greeks in a single call (price, delta, gamma, vega, theta)
- [ ] **BLK-04**: User can compute discounted option price directly
- [ ] **BLK-05**: Library provides PricingError type with AboveMaximum, BelowIntrinsic, InvalidInput variants

### Implied Volatility

- [ ] **IVOL-01**: User can compute Black implied volatility from option price (main entry point)
- [ ] **IVOL-02**: User can compute normalised implied volatility from normalised price
- [ ] **IVOL-03**: Library provides normalised Black call and normalised vega functions
- [ ] **IVOL-04**: Algorithm achieves machine-precision in 2 Householder iterations

## Future Requirements

### Calibration Integration

- **CALIB-01**: Calibration pipeline accepts raw option prices as input (uses IVOL to compute IVs)
- **CALIB-02**: eSSVI model implementation

## Out of Scope

| Feature | Reason |
|---------|--------|
| OptionType/CallPut enum | essvi doesn't have one; use i32 q (+1/-1) convention from Let's Be Rational directly |
| Black-Scholes (spot) model | Black-76 (futures) is sufficient for SVI calibration context |
| Real-time pricing | Library is for calibration, not live trading |
| Discounting framework | Simple exp(-rT) inline; no yield curve infrastructure |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| MATH-01 | Phase 6 | Done |
| MATH-02 | Phase 6 | Done |
| MATH-03 | Phase 6 | Done |
| MATH-04 | Phase 8 | Pending |
| MATH-05 | Phase 6 | Done |
| BLK-01 | Phase 7 | Pending |
| BLK-02 | Phase 7 | Pending |
| BLK-03 | Phase 7 | Pending |
| BLK-04 | Phase 7 | Pending |
| BLK-05 | Phase 7 | Pending |
| IVOL-01 | Phase 8 | Pending |
| IVOL-02 | Phase 8 | Pending |
| IVOL-03 | Phase 8 | Pending |
| IVOL-04 | Phase 8 | Pending |

**Coverage:**
- v1.1 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0

---
*Requirements defined: 2026-03-07*
*Last updated: 2026-03-07 after roadmap creation*
