# Roadmap: essvi

## Milestones

- v1.0 Idiomatic Restructuring - Phases 1-5 (shipped 2026-03-07)
- v1.1 Pricing Primitives - Phases 6-8 (shipped 2026-03-07)
- v1.2 Market Data Collection - Phases 9-11 (in progress)

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

### v1.2 Market Data Collection (In Progress)

**Milestone Goal:** Collect and store real European-style index option price data for testing and validation of the full calibration pipeline.

- [ ] **Phase 9: Data Schema and Directory Structure** - Define canonical CSV schema and create source-first directory layout
- [x] **Phase 10: Data Acquisition** - Download real option chain data for at least 2 indices across multiple dates (completed 2026-03-07)
- [x] **Phase 11: Provenance and Quality Documentation** - Document data sources, confirm exercise styles, and note quality issues (completed 2026-03-07)

## Phase Details

### Phase 9: Data Schema and Directory Structure
**Goal**: A well-defined data storage layout exists with clear conventions for all future data files
**Depends on**: Phase 8 (v1.1 complete)
**Requirements**: STOR-01, STOR-02, STOR-03
**Success Criteria** (what must be TRUE):
  1. `data/` directory exists with source-first hierarchy (`data/{source}/{underlying}/`)
  2. A canonical CSV schema is documented with all required and optional columns defined (strike, expiry, option_type, bid, ask, underlying_price, and optional forward, discount_factor, volume, open_interest, implied_vol)
  3. `data/README.md` exists as a data dictionary with column definitions, units, and file naming conventions
**Plans**: TBD

Plans:
- [ ] 09-01: TBD

### Phase 10: Data Acquisition
**Goal**: Real European-style index option chain data is stored in the defined structure
**Depends on**: Phase 9
**Requirements**: DATA-01, DATA-02, DATA-03
**Success Criteria** (what must be TRUE):
  1. At least 2 observation dates of option chain data exist for one index
  2. Data from at least 2 different European-style indices is present (e.g., Nikkei 225 + SPX, or Euro Stoxx 50 + SPX)
  3. Each data file contains both calls and puts across multiple expiry slices
  4. All CSV files conform to the canonical schema defined in Phase 9
**Plans**: TBD

Plans:
- [ ] 10-01: TBD

### Phase 11: Provenance and Quality Documentation
**Goal**: Every data file has documented origin, confirmed exercise style, and known quality issues
**Depends on**: Phase 10
**Requirements**: DOCS-01, DOCS-02, DOCS-03
**Success Criteria** (what must be TRUE):
  1. Each data file has source provenance recorded (URL, download date, collection method)
  2. Exercise style is confirmed as European for each index with exchange specification references
  3. Data quality notes exist documenting any known issues, excluded rows, or observations
**Plans**: TBD

Plans:
- [ ] 11-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 9 -> 10 -> 11

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
| 9. Data Schema and Directory Structure | v1.2 | 0/1 | Not started | - |
| 10. Data Acquisition | 1/1 | Complete    | 2026-03-07 | - |
| 11. Provenance and Quality Documentation | 1/1 | Complete    | 2026-03-07 | - |
