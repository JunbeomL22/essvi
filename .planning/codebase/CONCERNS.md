# Codebase Concerns

**Analysis Date:** 2026-03-07

## Tech Debt

**Massive code duplication across binaries:**
- Issue: `SliceData` struct, `make_slice()`, `build_market_slices()`, `FitResult` struct, `plot_fit()`, and weighting logic are copy-pasted across `src/bin/fit_real.rs`, `src/bin/fit_real_surface.rs`, and `src/bin/report.rs`. The `make_slice()` function alone is ~35 lines duplicated verbatim between `fit_real.rs` (lines 60-104) and `fit_real_surface.rs` (lines 24-58). `build_market_slices()` is duplicated identically. `plot_fit()` is near-identical across all three binaries with only minor title string differences.
- Files: `src/bin/fit_real.rs`, `src/bin/fit_real_surface.rs`, `src/bin/report.rs`
- Impact: Any bug fix or improvement to slice generation, plotting, or fit-result computation must be applied in three places. Divergence risk is high -- `fit_real_surface.rs` already has extra fields (`calendar_violations`, `max_calendar_violation_bps`) while the other two do not, making it easy for shared logic to drift apart silently.
- Fix approach: Extract shared types and functions into a library module (e.g., `src/market_data.rs` for `SliceData`, `make_slice`, `build_market_slices`; `src/plotting.rs` for `plot_fit`; `src/fit_result.rs` for `FitResult` and `compute_fit_result`). Binaries should import from the library.

**`plotters` is a non-dev dependency used only in binaries:**
- Issue: The `plotters` crate (line 7 of `Cargo.toml`) is listed as a regular dependency, but it is only used in `src/bin/*.rs` for report generation. Any downstream consumer of the `essvi` library crate will pull in `plotters` and its transitive dependencies (including font rendering, image codecs, etc.) even if they only need the calibration math.
- Files: `Cargo.toml` (line 7), `src/bin/fit_real.rs`, `src/bin/fit_real_surface.rs`, `src/bin/report.rs`
- Impact: Unnecessarily inflated compile times and dependency tree for library users. The core library (`src/lib.rs`) has zero dependencies, which is a strength that gets undermined by bundling `plotters` unconditionally.
- Fix approach: Move `plotters` behind a cargo feature flag (e.g., `[features] plotting = ["plotters"]`) and gate the binaries on that feature, or simply move plotting into a separate workspace member/crate.

**No custom error type -- `Option<T>` used as the sole error signaling mechanism:**
- Issue: `calibrate()` and `calibrate_with_calendar_penalty()` return `Option<CalibrationResult>`. `solve_theta()` returns `Option<f64>`. When calibration fails, the caller gets `None` with zero diagnostic information -- no indication of whether failure was due to no-arbitrage constraint violation, Newton divergence, boundary saturation, or max iterations exceeded.
- Files: `src/calibration.rs` (lines 112, 174, 209, 244)
- Impact: Debugging calibration failures in production requires guesswork. The `fit_real_surface.rs` binary (line 310) calls `.expect("first slice must calibrate")` which will panic with no useful context if the first slice fails.
- Fix approach: Define a `CalibrationError` enum with variants like `NewtonDivergence`, `NoArbitrageSaturation`, `MaxIterations`, `InvalidInput`. Return `Result<CalibrationResult, CalibrationError>` instead of `Option`.

**Magic numbers scattered throughout calibration code:**
- Issue: Numerical constants like `1e10` (penalty for infeasible points), `1e-6` (parameter bounds), `1e-14` (Newton tolerance), `1e-30` (derivative guard), `1e-15` (k_star threshold), `0.05` (initial simplex perturbation factor), `20` (rho grid points), `100.0` (calendar penalty lambda), and `3.0` / `1.0` (ATM/wing weights) appear as bare literals without named constants or documentation of why those specific values were chosen.
- Files: `src/calibration.rs` (lines 24, 30, 52, 113-114, 120, 129, 167-168), `src/nelder_mead.rs` (lines 63-66), `src/bin/fit_real.rs` (lines 132-133), `src/bin/fit_real_surface.rs` (lines 233, 247-249, 296-297)
- Impact: Difficult to tune or understand the sensitivity of the calibration to these choices. Someone modifying the rho grid density (currently 20 steps) has no way to know if this is a carefully tested value or an arbitrary choice.
- Fix approach: Define named constants with doc comments explaining the rationale, or promote them to `NelderMeadConfig`/`CalibrationConfig` fields with documented defaults.

## Known Bugs

**NaN propagation causes panic in Nelder-Mead sort:**
- Symptoms: `partial_cmp().unwrap()` on line 82 of `src/nelder_mead.rs` will panic if any function evaluation returns `NaN`. Since `f64::NAN.partial_cmp(&_)` returns `None`, the `.unwrap()` panics.
- Files: `src/nelder_mead.rs` (line 82)
- Trigger: If the objective function returns `NaN` (possible when `theta.powf(gamma)` receives negative theta due to floating point drift, or when `disc.sqrt()` receives a negative discriminant due to `rho` very close to +/-1), the simplex sort panics instead of recovering gracefully.
- Workaround: The objective functions in `src/calibration.rs` return `1e10` for invalid parameters, which prevents most NaN paths. But edge cases with extreme parameter values during simplex operations could still reach this panic.

**`solve_theta` Newton iteration can overshoot to negative theta without guard:**
- Symptoms: Newton update `theta -= residual / dw` at line 56 of `src/calibration.rs` can produce negative `theta`. The guard at line 33 (`if theta <= 0.0 { return None }`) catches this, but only at the start of the next iteration. Between the update and the guard, the negative theta is not used -- but the function returns `None` rather than attempting recovery (e.g., bisection fallback).
- Files: `src/calibration.rs` (lines 33, 56-57)
- Trigger: Very steep skews or near-zero `theta_star` values where the Newton step overshoots.
- Workaround: The calibration objective catches `None` and returns `1e10`, so the optimizer avoids these regions. But it means the optimizer may fail to explore valid parameter space near the boundary.

## Security Considerations

**No security-relevant attack surface:**
- Risk: This is a pure numerical computation library with no network, filesystem (beyond binaries writing reports), or user input handling. No secrets, authentication, or external service calls.
- Files: N/A
- Current mitigation: N/A
- Recommendations: If this library is ever exposed via a web API, validate input array lengths (prevent excessive allocations from huge `k_slice` arrays) and add timeouts on calibration calls (Nelder-Mead with `max_iter=1000` and 21 rho grid points runs ~21000 inner optimizer iterations per `calibrate` call).

## Performance Bottlenecks

**Rho grid sweep performs 21 independent 2D optimizations sequentially:**
- Problem: `calibrate()` in `src/calibration.rs` (lines 119-146) runs a Nelder-Mead optimization for each of 21 rho grid points. Each 2D optimization runs up to 1000 iterations, each calling `solve_theta` (up to 20 Newton steps) plus `total_variance_slice`. This is the dominant cost of calibration.
- Files: `src/calibration.rs` (lines 119-146)
- Cause: Serial loop over rho grid with no parallelism. Each rho grid point is fully independent.
- Improvement path: Use `rayon::par_iter` to parallelize the rho grid sweep. The objective closure captures only `&input` (shared reference) so parallelism is straightforward. Expected ~4-8x speedup on multicore machines for single-slice calibration.

**Surface calibration doubles the work by re-running unconstrained fit as initial guess:**
- Problem: In `src/bin/fit_real_surface.rs` (line 132) and `benches/calibration.rs` (line 132), the surface calibration first runs a full unconstrained `calibrate()` to get an initial guess, then runs `calibrate_with_calendar_penalty()` starting from that guess. This doubles the wall time for every slice after the first.
- Files: `src/bin/fit_real_surface.rs` (lines 258-283, 286-339), `benches/calibration.rs` (lines 128-134)
- Cause: `calibrate_with_calendar_penalty()` does not include its own rho grid sweep -- it takes an initial point and runs a single 3D Nelder-Mead. The unconstrained fit provides that initial point.
- Improvement path: Cache unconstrained results rather than recomputing them in the surface pass (the binary already does this), or add rho grid sweep to `calibrate_with_calendar_penalty()` itself.

**Heap allocations in hot loop (total_variance_slice returns Vec):**
- Problem: `ssvi::total_variance_slice()` allocates a new `Vec<f64>` on every call. During calibration, this function is called inside the Nelder-Mead objective, which itself runs inside a 21-point rho sweep. Total: ~21 * 1000 * 1 = ~21,000 heap allocations per `calibrate()` call.
- Files: `src/ssvi.rs` (lines 20-31)
- Cause: The function signature returns `Vec<f64>` rather than writing into a pre-allocated buffer.
- Improvement path: Add `total_variance_slice_into(k_slice: &[f64], ..., out: &mut [f64])` that writes into a caller-provided buffer, reused across iterations. The objective closure in `calibrate()` could hold a single `Vec<f64>` buffer.

## Fragile Areas

**Nelder-Mead parameter naming collision with SSVI model parameters:**
- Files: `src/nelder_mead.rs` (lines 4-12)
- Why fragile: `NelderMeadConfig` uses field names `gamma`, `rho`, and `sigma` -- identical to SSVI model parameters (gamma, rho) and the implied volatility symbol (sigma). This creates constant cognitive overhead and high risk of confusion when reading calibration code that uses both `config.rho` (contraction coefficient = 0.5) and the SSVI `rho` (skew parameter in [-1, 1]).
- Safe modification: Rename Nelder-Mead config fields to their standard textbook names with prefixes: `nm_alpha`, `nm_gamma`, `nm_rho`, `nm_sigma` -- or use the less common but unambiguous names like `contraction`, `expansion`, `reflection`, `shrink`.
- Test coverage: Adequate (Rosenbrock and boundary tests), but renaming would not break the API since it is a config struct.

**ATM identification in binaries uses midpoint index, not actual ATM:**
- Files: `src/bin/fit_real.rs` (lines 128-130), `src/bin/fit_real_surface.rs` (lines 243-245)
- Why fragile: `slice.k[slice.k.len() / 2]` assumes the midpoint of the k-array is near ATM (k=0). If the k-array is not centered around zero (which is the case -- many slices have asymmetric ranges like `[-0.55, 0.45]`), the "ATM" reference point is biased. The midpoint of 60 points spanning `[-0.55, 0.45]` is `k ~ -0.05`, not `k=0`.
- Safe modification: Find the k-point closest to 0.0, or interpolate to get the exact ATM total variance at k=0.
- Test coverage: No test verifies that theta_star/k_star are correctly identified from market data.

**No input validation on calibration inputs:**
- Files: `src/calibration.rs` (lines 90-96, 112)
- Why fragile: `CalibrationInput` does not validate that `k_slice` and `w_market` have the same length, that `w_market` values are positive, or that `weights` (if provided) matches the slice length. Mismatched lengths would produce silent incorrect results via `zip` (truncating to the shorter length).
- Safe modification: Add assertions or return `Err` for mismatched lengths at the start of `calibrate()`.
- Test coverage: No test with mismatched input lengths.

## Scaling Limits

**Single-threaded calibration limits throughput:**
- Current capacity: ~400 single-slice calibrations per second (based on benchmark: ~2.5ms per 20-point slice). A 12-slice surface fit takes ~90ms.
- Limit: For real-time pricing with hundreds of underlyings each having 10-20 expiry slices, serial calibration would require ~0.5-1 second per underlying, which is too slow for live trading.
- Scaling path: Parallelize rho grid sweep with `rayon`, and parallelize independent slice calibrations across underlyings. The code is already free of global mutable state, making this straightforward.

## Dependencies at Risk

**`plotters` crate as the sole visualization dependency:**
- Risk: Low severity. `plotters` is well-maintained but adds significant compile-time cost. It is only used in binaries, not the library.
- Impact: No impact on core library functionality. Binary compile times increase.
- Migration plan: Already addressed in tech debt above (feature-gate it).

**Rust edition 2024 requirement:**
- Risk: `Cargo.toml` specifies `edition = "2024"`, which requires Rust 1.85+. Users on older stable toolchains cannot compile this crate.
- Impact: Limits adoption. Edition 2024 provides no features that this codebase uses that are unavailable in edition 2021.
- Migration plan: Consider downgrading to `edition = "2021"` unless edition-2024-specific features are intentionally used.

## Missing Critical Features

**No support for reading real market data:**
- Problem: All binaries use hardcoded synthetic data via `make_slice()` with deterministic pseudo-noise. There is no CSV/JSON reader, no market data adapter, and no way to calibrate to actual exchange-traded option prices without writing a new binary.
- Blocks: Cannot validate the model against real market data without first building a data ingestion pipeline.

**No butterfly/calendar arbitrage verification as a post-calibration check:**
- Problem: The no-arbitrage condition `eta * (1 + |rho|) <= 2` is checked during calibration as a constraint, but the full butterfly arbitrage condition (non-negative probability density) is never verified. Calendar arbitrage is checked only in `fit_real_surface.rs` at 21 discrete k-points, not as a continuous guarantee.
- Blocks: Cannot certify that calibrated parameters produce an arbitrage-free surface. The discrete check can miss narrow violations between sample points.

**No implied volatility to/from option price conversion:**
- Problem: The library operates entirely in total-variance space. There is no Black-Scholes pricer, no IV solver (despite `brent.rs` existing), and no forward/discount factor handling. Converting between implied volatility and option prices must be done externally.
- Blocks: Cannot be used as a standalone pricing/calibration tool without an external Black-Scholes implementation.

**`brent.rs` is unused in the codebase:**
- Problem: `src/brent.rs` implements Brent's root-finding method and is exported via `src/lib.rs`, but it is never imported or called anywhere in the library or binaries. It appears to have been written for future IV solving but is currently dead code.
- Files: `src/brent.rs`, `src/lib.rs` (line 1)
- Blocks: Nothing -- but it adds maintenance burden and may mislead users into thinking it is used internally.

## Test Coverage Gaps

**No tests for `calibrate_with_calendar_penalty()`:**
- What's not tested: The calendar-penalized calibration function has zero unit tests. It is only exercised indirectly by the `fit_real_surface` binary and the benchmark.
- Files: `src/calibration.rs` (lines 209-253)
- Risk: Regressions in the calendar penalty logic (e.g., sign errors in the penalty term, lambda scaling issues) would go undetected. The function has a different calling convention than `calibrate()` (takes `init` parameter, no rho grid sweep) and its correctness depends on the penalty function working correctly.
- Priority: High

**No tests for edge cases in numerical routines:**
- What's not tested: `solve_theta` with extreme inputs (very large theta_star, very small theta_star near machine epsilon, rho near +/-1, gamma near 0 or 1). `nelder_mead_bounded` with degenerate bounds (lb == ub), zero-dimensional input, or NaN-producing objectives. `brent` with functions that have roots at the boundary endpoints.
- Files: `src/calibration.rs`, `src/nelder_mead.rs`, `src/brent.rs`
- Risk: Numerical failures in production with real market data that exercises parameter corners not covered by synthetic test data. The steep_skew test (`tests/steep_skew.rs`) is good but does not assert on parameter recovery, only that calibration succeeds.
- Priority: High

**No tests for binary logic (`src/bin/*.rs`):**
- What's not tested: ATM identification (midpoint assumption), weight construction, IV-to-total-variance conversion, fit error computation, report generation, plot generation.
- Files: `src/bin/fit_real.rs`, `src/bin/fit_real_surface.rs`, `src/bin/report.rs`
- Risk: Medium -- these are report-generation tools, not production code. But the ATM identification bug (midpoint != ATM) affects calibration quality.
- Priority: Medium

**No property-based or fuzz testing:**
- What's not tested: Random parameter combinations that might trigger NaN, infinity, or panic in the numerical routines.
- Files: All `src/*.rs`
- Risk: Undiscovered edge cases in floating-point arithmetic. The `partial_cmp().unwrap()` panic in Nelder-Mead (line 82 of `src/nelder_mead.rs`) is a known example of a defect that property-based testing would likely find.
- Priority: Medium

---

*Concerns audit: 2026-03-07*
