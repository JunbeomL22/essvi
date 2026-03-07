# Milestones

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

