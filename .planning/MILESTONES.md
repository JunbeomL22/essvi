# Milestones

## v1.0 Idiomatic Restructuring (Shipped: 2026-03-07)

**Delivered:** Transformed essvi from a working-but-rough codebase into idiomatic Rust with clean module hierarchy, proper error types, configurable calibration, deduplicated binaries, and external tests.

**Phases completed:** 5 phases, 5 plans
**Requirements:** 23/23 complete (CONF x6, STRC x6, API x5, TEST x6)
**Lines of Rust:** 2,284
**Git range:** 2726c8b..5bb3e21
**Timeline:** 2 days (2026-03-05 to 2026-03-07)

**Key accomplishments:**
- Reorganized flat module structure into `solver/` and `model/` submodule hierarchy with backward-compatible re-exports
- Replaced `Option<T>` error handling with `Result<T, CalibError>` across entire calibration API (4 error variants)
- Extracted all hardcoded calibration constants into `CalibrationConfig` struct with `Default` impl (12 configurable fields)
- Eliminated ~260 lines of duplicated code between binaries via `fit_common` library module
- Migrated all 12 inline unit tests to `tests/` directory; zero `#[cfg(test)]` blocks remain in `src/`

---

