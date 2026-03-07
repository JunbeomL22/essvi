# Roadmap: essvi

## Milestones

- v1.0 Idiomatic Restructuring - Phases 1-5 (shipped 2026-03-07)
- v1.1 Pricing Primitives - Phases 6-8 (shipped 2026-03-07)
- v1.2 Market Data Collection - Phases 9-11 (shipped 2026-03-07)
- v1.3 Crude Solver - Phases 12-14 (in progress)

## Phases

<details>
<summary>v1.0 Idiomatic Restructuring (Phases 1-5) - SHIPPED 2026-03-07</summary>

- [x] Phase 1: Module Restructuring (1/1 plans) - completed 2026-03-07
- [x] Phase 2: Error Types and Impl Blocks (1/1 plans) - completed 2026-03-07
- [x] Phase 3: Calibration Config (1/1 plans) - completed 2026-03-07
- [x] Phase 4: Binary Deduplication (1/1 plans) - completed 2026-03-07
- [x] Phase 5: Test Migration (1/1 plans) - completed 2026-03-07

</details>

<details>
<summary>v1.1 Pricing Primitives (Phases 6-8) - SHIPPED 2026-03-07</summary>

- [x] Phase 6: Math Foundations (1/1 plans) - completed 2026-03-07
- [x] Phase 7: Black-76 Pricing (1/1 plans) - completed 2026-03-07
- [x] Phase 8: Implied Volatility Solver (1/1 plans) - completed 2026-03-07

</details>

<details>
<summary>v1.2 Market Data Collection (Phases 9-11) - SHIPPED 2026-03-07</summary>

- [x] Phase 9: Data Schema and Directory Structure (1/1 plans) - completed 2026-03-07
- [x] Phase 10: Data Acquisition (1/1 plans) - completed 2026-03-07
- [x] Phase 11: Provenance and Quality Documentation (1/1 plans) - completed 2026-03-07

</details>

### v1.3 Crude Solver (Phases 12-14)

- [ ] **Phase 12: Crude Solver Core** - CrudeCalibConfig struct and single-slice 4D Nelder-Mead calibration
- [ ] **Phase 13: Sequential Calibration** - Multi-slice sequential calibration with calendar spread penalty
- [ ] **Phase 14: Crude Solver Binary** - Binary to run crude_solver on real data with SVG fit plots

## Phase Details

### Phase 12: Crude Solver Core
**Goal**: A single volatility slice can be calibrated by directly optimizing all four SSVI parameters (theta, eta, gamma, rho) via bounded Nelder-Mead
**Depends on**: Nothing (uses existing nelder_mead and ssvi modules)
**Requirements**: CRUD-01, CRUD-02
**Success Criteria** (what must be TRUE):
  1. CrudeCalibConfig struct exists with theta bounds, 4D Nelder-Mead tolerances, lambda, and k_penalty grid settings, and has a Default impl
  2. Calling crude_solver::calibrate_slice with market data returns a CalibrationResult with (theta, eta, gamma, rho) that minimizes SSE on total variance
  3. The objective function penalizes no-arbitrage butterfly violations, producing fits that satisfy the butterfly constraint on the fitted k-grid
  4. All existing tests (111) continue to pass with no regressions
**Plans**: TBD

### Phase 13: Sequential Calibration
**Goal**: Multiple volatility slices across expiries can be calibrated sequentially with monotonic theta enforcement via calendar spread penalty
**Depends on**: Phase 12
**Requirements**: CRUD-03
**Success Criteria** (what must be TRUE):
  1. Slices are sorted by expiry T and calibrated from shortest to longest
  2. For each slice after the first, the objective function includes a lambda-weighted penalty term that penalizes theta values below the previous slice's fitted theta
  3. The resulting theta sequence across slices is monotonically non-decreasing (or nearly so, with violations bounded by lambda strength)
**Plans**: TBD

### Phase 14: Crude Solver Binary
**Goal**: User can run a single command to calibrate real SPX/NDX option chains with crude_solver and inspect fit quality via SVG plots
**Depends on**: Phase 12, Phase 13
**Requirements**: BIN-01, BIN-02
**Success Criteria** (what must be TRUE):
  1. Running `cargo run --bin fit_crude` (or equivalent) parses CSV market data from data/cboe/ and calibrates all slices
  2. The binary outputs per-slice SVG plots showing model total variance vs market total variance across log-moneyness
  3. The binary prints per-slice fit summary (T, theta, eta, gamma, rho, SSE) to stdout
  4. The binary handles both SPX and NDX data without code changes (parameterized by path or argument)
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Module Restructuring | v1.0 | 1/1 | Complete | 2026-03-07 |
| 2. Error Types and Impl Blocks | v1.0 | 1/1 | Complete | 2026-03-07 |
| 3. Calibration Config | v1.0 | 1/1 | Complete | 2026-03-07 |
| 4. Binary Deduplication | v1.0 | 1/1 | Complete | 2026-03-07 |
| 5. Test Migration | v1.0 | 1/1 | Complete | 2026-03-07 |
| 6. Math Foundations | v1.1 | 1/1 | Complete | 2026-03-07 |
| 7. Black-76 Pricing | v1.1 | 1/1 | Complete | 2026-03-07 |
| 8. Implied Volatility Solver | v1.1 | 1/1 | Complete | 2026-03-07 |
| 9. Data Schema and Directory Structure | v1.2 | 1/1 | Complete | 2026-03-07 |
| 10. Data Acquisition | v1.2 | 1/1 | Complete | 2026-03-07 |
| 11. Provenance and Quality Documentation | v1.2 | 1/1 | Complete | 2026-03-07 |
| 12. Crude Solver Core | v1.3 | 0/? | Not started | - |
| 13. Sequential Calibration | v1.3 | 0/? | Not started | - |
| 14. Crude Solver Binary | v1.3 | 0/? | Not started | - |
