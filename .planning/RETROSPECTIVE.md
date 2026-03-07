# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — Idiomatic Restructuring

**Shipped:** 2026-03-07
**Phases:** 5 | **Plans:** 5 | **Sessions:** 1

### What Was Built
- Module hierarchy (`solver/`, `model/`) with backward-compatible re-exports
- `CalibError` enum with 4 variants replacing `Option<T>` across all calibration APIs
- `CalibrationConfig` struct with 12 configurable fields and `Default` impl
- `fit_common` shared library module eliminating ~260 lines of binary duplication
- External test suite (4 test files in `tests/`) with zero inline test code remaining

### What Worked
- YOLO mode completed all 5 phases in a single automated session with zero failures
- Each phase was small enough for a single plan — no multi-plan coordination overhead
- Sequential phase ordering (1→2→3→4→5) avoided rework; signature changes in Phase 2/3 happened before Phase 4's dedup
- Backward-compatible re-exports in Phase 1 meant downstream code changes were minimal

### What Was Inefficient
- Phase directories were not created by YOLO subagents, so SUMMARY.md/VERIFICATION.md artifacts are absent — milestone stats had to be reconstructed from git
- Phase 4 plan explored 5 options (A-E) for binary code sharing before settling on the library module approach — could have been decided upfront in roadmap

### Patterns Established
- For small Rust crates, one plan per phase is sufficient when phases are well-scoped
- Shared binary code belongs in a library module (`src/fit_common.rs`), not `src/bin/` (Cargo auto-discovery)
- FitResult superset pattern (default zeros for optional fields) is cleaner than Option wrapping

### Key Lessons
1. Scope phases to single-plan granularity for YOLO runs — multi-plan phases risk context exhaustion
2. Document design decisions (like "where to put shared binary code") in the roadmap phase details, not during execution
3. `CalibrationConfig` with `Default` is the right pattern for Rust calibration APIs — keeps API surface small while allowing customization

### Cost Observations
- Model mix: 100% opus (quality profile)
- Sessions: 1 (full YOLO run)
- Notable: 5 phases completed in ~24 minutes of agent time total

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | 1 | 5 | Initial milestone, YOLO mode, single-plan phases |

### Cumulative Quality

| Milestone | Tests | Coverage | Zero-Dep Additions |
|-----------|-------|----------|-------------------|
| v1.0 | 14 | Unit + integration | CalibError, CalibrationConfig |

### Top Lessons (Verified Across Milestones)

1. (Awaiting second milestone for cross-validation)
