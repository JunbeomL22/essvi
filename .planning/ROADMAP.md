# Roadmap: essvi v1.0 Idiomatic Restructuring

## Overview

Transform the essvi library from a working-but-rough codebase into idiomatic Rust: reorganize flat modules into solver/ and model/ hierarchies, replace Option-based error handling with proper CalibError types, extract hardcoded constants into a CalibrationConfig struct, deduplicate shared binary code, and migrate all inline tests to the tests/ directory. Each phase builds on the previous -- module moves create the foundation, error types change signatures, config changes parameters, binary dedup leverages the new structure, and test migration validates everything still works.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Module Restructuring** - Move source files into solver/ and model/ submodule hierarchy
- [x] **Phase 2: Error Types and Impl Blocks** - Replace Option returns with Result<T, CalibError> and add methods to domain structs
- [x] **Phase 3: Calibration Config** - Extract hardcoded constants into CalibrationConfig struct with Default impl
- [ ] **Phase 4: Binary Deduplication** - Extract shared code from fit_real binaries into a common module
- [ ] **Phase 5: Test Migration** - Move all inline tests to tests/ directory and verify coverage

## Phase Details

### Phase 1: Module Restructuring
**Goal**: Library source is organized into clear solver/ and model/ submodule directories with proper re-exports
**Depends on**: Nothing (first phase)
**Requirements**: STRC-01, STRC-02, STRC-03, STRC-04
**Success Criteria** (what must be TRUE):
  1. `use essvi::solver::nelder_mead` and `use essvi::solver::brent` resolve correctly
  2. `use essvi::model::ssvi` resolves correctly
  3. `cargo build` succeeds with zero warnings about the module restructuring
  4. All three binaries (report, fit_real, fit_real_surface) compile and run against the new module paths
**Plans**: Completed (1 plan: move files, create mod.rs, update imports, add re-exports)

### Phase 2: Error Types and Impl Blocks
**Goal**: Callers can distinguish failure modes via CalibError variants and access convenience methods on CalibrationResult
**Depends on**: Phase 1
**Requirements**: API-01, API-02, API-03, API-04, API-05
**Success Criteria** (what must be TRUE):
  1. CalibrationResult exposes phi() and no_arb_usage() methods that return computed values without manual field access
  2. solve_theta() returns Result<f64, CalibError> with distinct error variants for divergence, non-positive theta, and zero derivative
  3. calibrate() returns Result<CalibrationResult, CalibError> with a non-convergence variant
  4. calibrate_with_calendar_penalty() returns Result<CalibrationResult, CalibError>
  5. Existing binary callers handle the new Result returns (compile and run without panics)
**Plans**: Completed (1 plan: define CalibError enum, add CalibrationResult impl block, convert solve_theta/calibrate/calibrate_with_calendar_penalty to Result, update all callers)

### Phase 3: Calibration Config
**Goal**: All calibration tuning knobs live in a single CalibrationConfig struct with sensible defaults, eliminating hardcoded constants
**Depends on**: Phase 2
**Requirements**: CONF-01, CONF-02, CONF-03, CONF-04, CONF-05, CONF-06
**Success Criteria** (what must be TRUE):
  1. CalibrationConfig::default() returns the same values currently hardcoded in the source (eta/gamma/rho bounds, n_rho=20, rho sweep, k_penalty range, lambda=100, solver tolerances)
  2. calibrate() accepts a CalibrationConfig parameter and uses its values for bounds, grid, and tolerances
  3. calibrate_with_calendar_penalty() accepts CalibrationConfig for penalty parameters
  4. solve_theta() respects tolerance and max_iter from CalibrationConfig
  5. A user can construct a custom CalibrationConfig, pass it to calibrate(), and observe different optimization behavior (e.g., narrower rho sweep)
**Plans**: Completed (1 plan: define CalibrationConfig struct with Default, update solve_theta/calibrate/calibrate_with_calendar_penalty signatures, update all callers)

### Phase 4: Binary Deduplication
**Goal**: Shared code between fit_real.rs and fit_real_surface.rs lives in one place, eliminating duplication
**Depends on**: Phase 1
**Requirements**: STRC-05, STRC-06
**Success Criteria** (what must be TRUE):
  1. SliceData, make_slice, build_market_slices, FitResult, and plot_fit exist in exactly one shared module
  2. Both fit_real and fit_real_surface import from the shared module and produce identical output to before
**Plans**: TBD

### Phase 5: Test Migration
**Goal**: All unit tests live in the tests/ directory; source files contain zero test code
**Depends on**: Phase 2, Phase 3
**Requirements**: TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, TEST-06
**Success Criteria** (what must be TRUE):
  1. tests/ssvi.rs, tests/calibration.rs, tests/nelder_mead.rs, and tests/brent.rs exist and contain the migrated unit tests
  2. Zero #[cfg(test)] mod tests blocks remain in any src/ file
  3. `cargo test` passes all tests with the same pass/fail results as before migration
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5
(Phase 4 depends only on Phase 1, so it could run after Phase 1, but sequencing after Phase 3 avoids churn from signature changes.)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Module Restructuring | 1/1 | Complete | 2026-03-07 |
| 2. Error Types and Impl Blocks | 1/1 | Complete | 2026-03-07 |
| 3. Calibration Config | 1/1 | Complete | 2026-03-07 |
| 4. Binary Deduplication | 0/0 | Not started | - |
| 5. Test Migration | 0/0 | Not started | - |
