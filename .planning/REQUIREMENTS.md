# Requirements: essvi v1.0 Idiomatic Restructuring

**Defined:** 2026-03-07
**Core Value:** Accurate, arbitrage-free implied volatility surface calibration

## v1 Requirements

Requirements for v1.0 milestone. Each maps to roadmap phases.

### Configuration

- [ ] **CONF-01**: CalibrationConfig struct holds parameter bounds (eta, gamma, rho ranges), rho grid config (n_rho, sweep range), and solver tolerances
- [ ] **CONF-02**: CalibrationConfig holds calendar penalty config (k_penalty range, step, lambda)
- [ ] **CONF-03**: Default impl on CalibrationConfig returns current hardcoded values
- [ ] **CONF-04**: calibrate() accepts CalibrationConfig instead of relying on internal constants
- [ ] **CONF-05**: calibrate_with_calendar_penalty() accepts CalibrationConfig for penalty parameters
- [ ] **CONF-06**: solve_theta() tolerances and max_iter configurable via CalibrationConfig

### Structure

- [x] **STRC-01**: Nelder-Mead optimizer lives at src/solver/nelder_mead.rs
- [x] **STRC-02**: Brent root finder lives at src/solver/brent.rs
- [x] **STRC-03**: SSVI model lives at src/model/ssvi.rs
- [x] **STRC-04**: src/solver/mod.rs and src/model/mod.rs re-export public items
- [ ] **STRC-05**: Shared binary code extracted to common module (SliceData, make_slice, build_market_slices, FitResult, plot_fit)
- [ ] **STRC-06**: fit_real.rs and fit_real_surface.rs import from shared module instead of duplicating

### API

- [ ] **API-01**: CalibrationResult has impl block with phi() and no_arb_usage() methods
- [ ] **API-02**: CalibError enum with variants for theta divergence, non-positive theta, zero derivative, and non-convergence
- [ ] **API-03**: solve_theta() returns Result<f64, CalibError>
- [ ] **API-04**: calibrate() returns Result<CalibrationResult, CalibError>
- [ ] **API-05**: calibrate_with_calendar_penalty() returns Result<CalibrationResult, CalibError>

### Testing

- [ ] **TEST-01**: tests/ssvi.rs contains all SSVI model unit tests (phi_basic, atm_total_variance, no_arb)
- [ ] **TEST-02**: tests/calibration.rs contains all calibration unit tests (solve_theta_basic, calibrate_recovers_parameters, etc.)
- [ ] **TEST-03**: tests/nelder_mead.rs contains optimizer unit tests (rosenbrock_2d, solution_on_boundary)
- [ ] **TEST-04**: tests/brent.rs contains root finder unit tests (find_sqrt2, no_sign_change)
- [ ] **TEST-05**: All #[cfg(test)] mod tests blocks removed from src/ files
- [ ] **TEST-06**: cargo test passes with identical coverage after migration

## Future Requirements

### eSSVI Model

- **ESSVI-01**: eSSVI model implementation (coexists with SSVI)
- **ESSVI-02**: eSSVI calibration pipeline
- **ESSVI-03**: Comparative fit quality report (SSVI vs eSSVI)

## Out of Scope

| Feature | Reason |
|---------|--------|
| eSSVI implementation | Deferred to next milestone -- structural cleanup first |
| Surface calibration improvements | Future milestone |
| Real market data parsing | Future milestone |
| Crate publishing / API docs | Future milestone |
| Async/parallel calibration | Not needed for current use case |
| Builder pattern on configs | Default impl sufficient for v1.0 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONF-01 | Phase 3 | Pending |
| CONF-02 | Phase 3 | Pending |
| CONF-03 | Phase 3 | Pending |
| CONF-04 | Phase 3 | Pending |
| CONF-05 | Phase 3 | Pending |
| CONF-06 | Phase 3 | Pending |
| STRC-01 | Phase 1 | Done |
| STRC-02 | Phase 1 | Done |
| STRC-03 | Phase 1 | Done |
| STRC-04 | Phase 1 | Done |
| STRC-05 | Phase 4 | Pending |
| STRC-06 | Phase 4 | Pending |
| API-01 | Phase 2 | Pending |
| API-02 | Phase 2 | Pending |
| API-03 | Phase 2 | Pending |
| API-04 | Phase 2 | Pending |
| API-05 | Phase 2 | Pending |
| TEST-01 | Phase 5 | Pending |
| TEST-02 | Phase 5 | Pending |
| TEST-03 | Phase 5 | Pending |
| TEST-04 | Phase 5 | Pending |
| TEST-05 | Phase 5 | Pending |
| TEST-06 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 23 total
- Mapped to phases: 23
- Unmapped: 0

---
*Requirements defined: 2026-03-07*
*Last updated: 2026-03-07 after roadmap creation*
