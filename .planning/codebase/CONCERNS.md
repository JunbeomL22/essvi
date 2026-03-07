# Codebase Concerns

**Analysis Date:** 2026-03-08

## Tech Debt

**Massive code duplication across binary fitting tools:**
- Issue: `fit_cboe.rs`, `fit_kaggle.rs`, and `fit_crude.rs` contain near-identical `fit_surface()` and `report()` functions, copy-pasted with only minor parameter differences (column names, lambda values, step sizes). The diff between `fit_cboe.rs` (430 lines) and `fit_kaggle.rs` (419 lines) shows ~95% identical code with only column name casing differences (`dte` vs `DTE`, `c_iv` vs `C_IV`) and a few threshold tweaks.
- Files: `src/bin/fit_cboe.rs`, `src/bin/fit_kaggle.rs`, `src/bin/fit_crude.rs`, `src/bin/fit_real.rs`, `src/bin/fit_real_surface.rs`
- Impact: Any change to calibration workflow, error reporting, or reporting format must be replicated across 3-5 binaries. Bug fixes in one binary may not propagate to others. The `fit_kaggle.rs` doc comment says "lambda = 100" but the code uses `lambda = 500.0` (copy-paste error from `fit_cboe.rs`).
- Fix approach: Extract shared `parse_csv()`, `fit_surface()`, and `report()` into `src/fit_common.rs` with configuration structs that capture per-data-source differences (column name mappings, lambda, penalty grid). Each binary becomes a thin configuration wrapper (~30 lines).

**Duplicate `norm_inv_cdf` implementations:**
- Issue: `src/math/normal.rs` and `src/math/normal_hp.rs` contain identical `norm_inv_cdf()` functions using the same Acklam rational approximation coefficients and Halley refinement. The only difference is that `normal_hp.rs` calls `norm_cdf` from the HP module (with asymptotic expansion). The coefficient arrays are duplicated verbatim.
- Files: `src/math/normal.rs` (lines 52-129), `src/math/normal_hp.rs` (lines 106-178)
- Impact: Any coefficient correction must be applied in two places. Risk of drift between implementations.
- Fix approach: Extract the Acklam approximation into a shared helper that takes a `cdf_fn` parameter, or consolidate by having `normal::norm_inv_cdf` delegate to `normal_hp::norm_inv_cdf` since the HP version is strictly superior.

**Unused modules in the library:**
- Issue: `src/math/normal_hp.rs` and `src/pricing/rational_cubic.rs` are publicly exported but not used by any other module in the library or any binary. They are compiled and tested via doc-tests but serve no functional purpose in the current codebase.
- Files: `src/math/normal_hp.rs`, `src/pricing/rational_cubic.rs`
- Impact: Dead code adds maintenance burden and confuses understanding of the dependency graph. The `lets_be_rational.rs` implied vol solver does NOT use either module despite the rational_cubic module's doc comment claiming it supports "initial guess refinement" for the IV solver.
- Fix approach: Either wire these modules into the implied vol solver (as seemingly intended) or mark them with `#[cfg(feature = "...")]` to indicate they are not yet integrated.

**Compiler warnings in `report.rs` binary:**
- Issue: Three dead-code warnings: unused field `label` in `Scenario`, unused field `theta` in `FitResult`, unused function `plot_heatmap`. These indicate incomplete refactoring.
- Files: `src/bin/report.rs` (lines 12, 53, 185)
- Impact: Warning noise during builds. Suggests `plot_heatmap` was planned but never called from `main()`.
- Fix approach: Either use `plot_heatmap` in the reporting pipeline or remove it. Remove unused struct fields or prefix them with `_`.

**Hardcoded magic numbers in objectives:**
- Issue: The penalty barrier value `1e10` is used as a "big number" return value in calibration objectives when parameters violate no-arbitrage or theta solve fails (7 occurrences). This is fragile -- if market data SSE is very large, `1e10` might not be a sufficient barrier.
- Files: `src/calibration.rs` (lines 247, 252, 279, 284, 357, 362), `src/crude_solver.rs` (line 248)
- Impact: In extreme market conditions (e.g., very high volatility regimes), the barrier may be insufficient to prevent the optimizer from exploring infeasible regions.
- Fix approach: Use `f64::INFINITY` or a named constant like `INFEASIBLE_PENALTY`. Since Nelder-Mead comparisons use `partial_cmp`, infinity works correctly.

## Known Bugs

**`fit_kaggle.rs` doc comment contradicts code:**
- Symptoms: The doc comment on `fit_surface()` at line 114 says "k_penalty: -1.5 to 0.5 step 0.05, lambda = 100" but the code uses `k += 0.025` (line 126) and `lambda = 500.0` (line 129). The Step 2 print statement also says "step 0.05" (line 196) while the loop uses 0.025.
- Files: `src/bin/fit_kaggle.rs` (lines 114, 126, 129, 196)
- Trigger: Compare comments to code.
- Workaround: Code behavior is correct (uses 0.025 step and lambda=500). Comments are stale from copy-paste.

## Security Considerations

**Data file paths constructed from user input without sanitization:**
- Risk: All binary entry points construct file paths directly from CLI arguments (e.g., `format!("data/cboe/{}/{}.csv", ticker, date)`). While this is a local CLI tool with no network exposure, path traversal could read arbitrary CSV files.
- Files: `src/bin/fit_cboe.rs` (line 420), `src/bin/fit_kaggle.rs` (line 409), `src/bin/fit_crude.rs` (line 285)
- Current mitigation: These are local CLI binaries, not services. The risk is minimal.
- Recommendations: For a library context, validate that ticker/date contain only alphanumeric characters and hyphens.

**Minimal `.gitignore`:**
- Risk: The `.gitignore` only ignores `/target`. Data files (CSVs with real market data) are tracked in git. If this becomes a public repository, licensed market data could leak.
- Files: `.gitignore`
- Current mitigation: Repository appears private.
- Recommendations: Add `data/cboe/`, `data/kaggle/`, `data/eurex/` to `.gitignore` if the data is not intended for redistribution. Add `*.env` and other secrets patterns as a precaution.

## Performance Bottlenecks

**Crude solver multi-start is computationally expensive:**
- Problem: `calibrate_slice_with_prev()` runs 12 multi-start points (4 rho starts x 3 eta starts), each running bounded Nelder-Mead with `max_iter=5000`. For a surface with N slices, this means 12*N optimizer runs, each evaluating a 4D objective that includes a 50-point butterfly penalty grid.
- Files: `src/crude_solver.rs` (lines 305-326)
- Cause: Brute-force multi-start strategy without warm-starting from previous slice results.
- Improvement path: Use the previous slice's fitted parameters as one of the start points (already done for theta_init but not for the multi-start grid). Reduce the number of starts for later slices where the landscape is more predictable.

**Nelder-Mead sorts full simplex every iteration:**
- Problem: Every iteration sorts n+1 vertices by allocating new `Vec<Vec<f64>>` via `.clone()` and reconstructing both `simplex` and `fvals` arrays. For the 4D crude solver (5 vertices), this means 5 allocations per iteration x 5000 iterations = 25000 heap allocations per optimizer run.
- Files: `src/solver/nelder_mead.rs` (lines 81-86)
- Cause: The sort creates sorted copies via `order.iter().map(|&i| simplex[i].clone()).collect()` rather than sorting in-place.
- Improvement path: Sort indices and swap elements in-place, or track best/worst/second-worst indices directly (only 3 values needed) without full sorts.

**Butterfly density uses finite differences instead of analytic derivatives:**
- Problem: `butterfly_density()` computes w', w'' via central finite differences with h=1e-5, requiring 3 evaluations of `ssvi::total_variance` per k-point. The SSVI formula has closed-form first and second derivatives.
- Files: `src/crude_solver.rs` (lines 126-143)
- Cause: Simpler implementation using numerical derivatives.
- Improvement path: Implement analytic `dw_dk()` and `d2w_dk2()` in `src/model/ssvi.rs`. This would be approximately 3x faster for butterfly penalty evaluation and eliminate finite-difference truncation error.

## Fragile Areas

**Nelder-Mead `partial_cmp().unwrap()` on NaN-producing objectives:**
- Files: `src/solver/nelder_mead.rs` (line 82)
- Why fragile: If any objective function evaluation returns NaN (e.g., from `sqrt()` of a negative number), `partial_cmp().unwrap()` will panic. The SSVI total variance function can produce negative discriminants for extreme parameters, and while barriers at 1e10 are intended to prevent this, edge cases during simplex exploration (reflection, expansion) could bypass the barrier check.
- Safe modification: Replace `.unwrap()` with `.unwrap_or(std::cmp::Ordering::Equal)` or handle NaN function values explicitly by treating them as worst-case (infinity).
- Test coverage: No test explicitly checks NaN resilience of the optimizer.

**`partial_cmp().unwrap()` throughout sorting operations:**
- Files: `src/bin/fit_cboe.rs` (lines 100, 110, 147, 212), `src/bin/fit_kaggle.rs` (lines 97, 107, 144, 210), `src/bin/fit_crude.rs` (lines 124, 135), `src/crude_solver.rs` (line 289)
- Why fragile: Sorting f64 vectors by `partial_cmp().unwrap()` will panic if any value is NaN. Market data CSVs could contain NaN from parsing errors or missing data.
- Safe modification: Use `.unwrap_or(std::cmp::Ordering::Equal)` consistently.
- Test coverage: No test feeds NaN market data to parsers or calibrators.

**Calibration objective returns 1e10 for constraint violations rather than infinity:**
- Files: `src/calibration.rs` (lines 247, 252, 279, 284, 357, 362), `src/crude_solver.rs` (line 248)
- Why fragile: If market data total variance values happen to be large (e.g., during extreme volatility regimes), the SSE alone could approach or exceed 1e10, making the barrier ineffective at excluding infeasible parameter regions.
- Safe modification: Use `f64::INFINITY` instead of `1e10`. The Nelder-Mead optimizer handles infinity correctly via `partial_cmp`.
- Test coverage: No test exercises the optimizer with very large total variance inputs.

**Calendar violation detection uses magic epsilon `1e-14`:**
- Files: `src/bin/fit_cboe.rs` (line 279), `src/bin/fit_kaggle.rs` (line 276), `src/bin/fit_real_surface.rs` (line 41)
- Why fragile: The threshold `1e-14` for detecting `w_prev > w_cur` is hardcoded across multiple files. If the tolerance needs adjustment, all files must be updated manually.
- Safe modification: Define a named constant (e.g., `CALENDAR_ARB_TOLERANCE`) in `src/calibration.rs` or `src/fit_common.rs`.

## Scaling Limits

**Single-threaded calibration:**
- Current capacity: Sequential processing of all expiry slices. A typical surface with 15-20 slices takes a few seconds in release mode.
- Limit: The unconstrained Step 1 (per-slice) fits are independent and embarrassingly parallel, but the code processes them sequentially. The crude solver's 12 multi-start points per slice are also sequential.
- Scaling path: Use `rayon` for Step 1 parallel per-slice fitting. Multi-start points within crude solver could also be parallelized. Step 2 (sequential calendar penalty) must remain sequential by design.

**Fixed-size Nelder-Mead simplex initialization:**
- Current capacity: Initial simplex perturbation of 5% works well for 2D-4D problems.
- Limit: The 0.05 * x[j] perturbation and 0.00025 * (ub[j] - lb[j]) fallback are hardcoded. For parameters with very different scales (e.g., theta ~ 0.001 vs eta ~ 1.0), the initial simplex may be poorly shaped.
- Scaling path: Add per-dimension initial step sizes to `NelderMeadConfig`.

## Dependencies at Risk

**`plotters` as sole runtime dependency:**
- Risk: Low. `plotters` 0.3 is mature and actively maintained. However, it is a heavy dependency for a numerical library. It is used only in `src/fit_common.rs` (the `plot_fit` function) and the binary tools, but it is a top-level dependency, meaning any downstream user of the `essvi` library gets `plotters` transitively.
- Impact: Increased compile times, larger binary size, potential build issues on minimal platforms.
- Migration plan: Move `plotters` to a cargo feature flag (e.g., `[features] plotting = ["plotters"]`) and gate `fit_common::plot_fit` behind `#[cfg(feature = "plotting")]`. Binary tools can enable the feature.

## Missing Critical Features

**No error type for library API:**
- Problem: The library's public API (`calibrate`, `solve_theta`) returns `CalibError` but there is no unified error type for the library as a whole. `PricingError` and `CalibError` are separate, unrelated error enums with no common trait beyond `std::error::Error`.
- Blocks: Users cannot use `?` operator easily when mixing pricing and calibration operations. Composing implied vol computation with SSVI calibration requires manual error mapping.

**No serialization support:**
- Problem: `CalibrationResult`, `CalibrationConfig`, `CrudeCalibResult`, and `CrudeCalibConfig` lack `serde::Serialize`/`Deserialize` implementations. Results can only be consumed in-process or manually formatted (as the binaries do with markdown).
- Blocks: Cannot save/load calibration results to/from JSON, persist configurations, or integrate with web APIs.

**No input validation on calibration inputs:**
- Problem: `CalibrationInput` accepts raw slices without validating that `k_slice` and `w_market` have the same length, that `w_market` values are positive, or that `theta_star > 0`. Invalid inputs can cause silent numerical failures or panics in downstream operations.
- Blocks: Safe usage as a library where inputs come from external data sources.

## Test Coverage Gaps

**No tests for binary entry points or CSV parsing:**
- What's not tested: The `parse_cboe()`, `parse_kaggle()`, `fit_surface()`, and `report()` functions in all binary files are untested. CSV parsing logic with its 3-zone IV rule, DTE filtering, and edge case handling has zero test coverage.
- Files: `src/bin/fit_cboe.rs`, `src/bin/fit_kaggle.rs`, `src/bin/fit_crude.rs`, `src/bin/plot_kaggle.rs`
- Risk: Malformed CSVs, missing columns, or unexpected data formats will cause panics (via `.expect()` and `.unwrap()`) without any graceful error path. The 3-zone IV merge rule is a critical business logic decision that is untested.
- Priority: Medium -- the parsing code is copy-pasted across binaries and any bug affects multiple tools.

**No tests for the plotting function:**
- What's not tested: `fit_common::plot_fit()` is never tested. An empty `k` vector would cause a panic on `r.k.first().unwrap()` (line 139).
- Files: `src/fit_common.rs` (line 135)
- Risk: Plotting failures would crash the binary silently. The function does `.unwrap()` on `first()` and `last()` without checking for empty input.
- Priority: Low -- plotting is a presentation concern, not correctness-critical.

**No tests for `normal_hp` or `rational_cubic` modules:**
- What's not tested: These modules have doc-tests only. No integration tests verify the asymptotic expansion in `normal_hp::norm_cdf` for extreme arguments, or that `rational_cubic_interpolation` handles edge cases (equal endpoints, zero-width intervals).
- Files: `src/math/normal_hp.rs`, `src/pricing/rational_cubic.rs`
- Risk: Low since these modules are currently unused, but if wired into the IV solver in the future, edge cases could surface.
- Priority: Low (unused modules).

**No tests for weighted calibration:**
- What's not tested: All calibration tests use `weights: None`. The binary tools all use ATM-weighted calibration (`weights: Some(&weights)` with 3.0 for |k| < 0.2), but this code path is never tested.
- Files: `src/calibration.rs` (function `weighted_squared_error`), `tests/calibration.rs`
- Risk: The weighting logic could have bugs that only manifest in real-data fitting scenarios. The `weighted_squared_error` function itself is simple, but the interaction with optimizer convergence under weighting is untested.
- Priority: Medium -- weighted fitting is the default mode for all real-data usage.

**No tests for `calibrate_with_calendar_penalty`:**
- What's not tested: The calendar penalty variant used by surface fitting binaries. The crude solver has calendar penalty tests, but the main `calibration.rs` implementation of `calibrate_with_calendar_penalty` has zero test coverage.
- Files: `src/calibration.rs` (lines 343-387)
- Risk: Calendar penalty logic in the primary calibration pipeline could have bugs not caught by crude solver tests.
- Priority: High -- this is the primary production calibration path for surface fitting.

---

*Concerns audit: 2026-03-08*
