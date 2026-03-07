# Milestones

## v1.2 Market Data Collection (Shipped: 2026-03-07)

**Delivered:** Collected and documented real European-style index option chain data (SPX + NDX) with canonical CSV schema, source provenance, and quality notes.

**Phases completed:** 9-11 (3 plans total)
**Requirements:** 9/9 complete (STOR x3, DATA x3, DOCS x3)
**Data:** 33,680 CSV rows across 3 option chain files
**Git range:** 3431c6f..1e03957
**Timeline:** 2 days (2026-03-05 to 2026-03-07)

**Key accomplishments:**
- Created `data/` directory with source-first hierarchy and canonical CSV schema (12 columns across 3 tiers)
- Acquired 33,680 rows of real SPX and NDX option chain data via yfinance (47+ expiry slices each)
- Documented per-file source provenance with Yahoo Finance column mapping table
- Confirmed European exercise style for SPX and NDX with CBOE contract specification URLs
- Created comprehensive data quality notes covering duplicate data, empty volumes, high IV values, and bid=0 handling

**Stats:**
- 23 files created/modified
- 201 lines in data/README.md (data dictionary)
- 3 phases, 3 plans, 9 tasks
- 2 days from start to ship

**What's next:** v1.3 Data Parsing (CSV parser, put-call parity forward extraction, CalibrationInput conversion)

---

## v1.1 Pricing Primitives (Shipped: 2026-03-07)

**Delivered:** Ported Black-76 pricing and Let's Be Rational implied volatility solver as independent library modules with full math foundations (erf, normal distributions, rational cubic interpolation).

**Phases completed:** 3 phases, 3 plans
**Requirements:** 14/14 complete (MATH x5, BLK x5, IVOL x4)
**Lines of Rust:** 4,663 (up from 2,284)
**Git range:** dbd90cb..c8deaff
**Timeline:** 1 day (2026-03-07)

**Key accomplishments:**
- Implemented fdlibm-based erf/erfc/erfcx with machine-precision accuracy (Cody's rational Chebyshev approximations)
- Standard normal PDF, CDF, and inverse CDF via Acklam's algorithm with Halley refinement
- High-precision normal CDF with asymptotic tail expansion for extreme arguments
- Black-76 option pricing with full greeks (delta, gamma, vega, theta) and combined greeks call
- PricingError type with AboveMaximum, BelowIntrinsic, and InvalidInput variants
- Implied volatility solver achieving machine-precision convergence via bisection initial guess + Halley iterations

---

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

