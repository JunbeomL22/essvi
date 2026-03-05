# Codebase Concerns

**Analysis Date:** 2026-03-05

## Tech Debt

**No-Arbitrage Constraint Saturation in SSVI Model:**
- Issue: The SSVI no-arbitrage condition `eta * (1 + |rho|) <= 2` acts as a hard boundary that severely limits the model's ability to fit steep skew scenarios. When this constraint becomes saturated (approaches 2.0), the optimizer is constrained and fit quality degrades significantly. This is a fundamental limitation of the SSVI model that cannot be solved within the current architecture.
- Files: `src/ssvi.rs`, `src/calibration.rs` (lines 88-90, 114-116)
- Impact: For short-expiry (T < 0.1) and steep-skew markets (slope > 0.5), the model cannot produce the required skew intensity while maintaining no-arbitrage. Calibration succeeds but produces suboptimal fits with high IV errors, particularly on the put side (negative k).
- Fix approach: This is not a bug but a model limitation. Options: (1) accept the constraint and document maximum applicable market scenarios; (2) explore alternative smile models (e.g., SVI, WSVI) that don't have this constraint; (3) add pre-flight validation to reject inputs that would require saturation.

**Implicit Theta Solver Convergence Risk:**
- Issue: The `solve_theta()` function in `src/calibration.rs` (lines 10-39) uses Brent's method to solve the implicit ATM consistency equation. If the root-finding algorithm fails to converge, it returns `None`, which propagates as calibration failure. The bracketing interval `[theta_star/1000, 100*theta_star]` is hardcoded and may not contain the root in edge cases.
- Files: `src/calibration.rs` (lines 10-39, 31-33)
- Impact: Calibration can fail silently for certain parameter combinations, particularly with non-zero `k_star` values where the functional form is more complex. No diagnostic information is provided about why the solver failed.
- Fix approach: (1) Add logging/diagnostics to report solver failures; (2) make bracket bounds adaptive based on input parameters; (3) try multiple bracket strategies before giving up; (4) consider analytical solutions for special cases (e.g., `k_star=0`).

**Hard-Coded Tolerance and Iteration Limits:**
- Issue: Tolerances for Brent root-finding (`1e-14` in line 33 of `src/calibration.rs`) and Nelder-Mead optimization (`tol_f: 1e-12, tol_x: 1e-12` in `src/nelder_mead.rs` line 18-19) are fixed. These may be too tight for some problems and too loose for others, with no way to tune them without modifying source code.
- Files: `src/calibration.rs` (line 33), `src/nelder_mead.rs` (line 18-19)
- Impact: Users cannot control optimizer behavior without recompilation. Convergence time is unpredictable and could timeout on resource-constrained systems.
- Fix approach: Expose tolerance parameters through `NelderMeadConfig` and `solve_theta()` as optional arguments. Consider adaptive tolerancing based on problem scale.

**Numerical Instability in Phi Function:**
- Issue: The `phi()` function in `src/ssvi.rs` (line 6) computes `eta / (theta.powf(gamma) * (1.0 + theta).powf(1.0 - gamma))`. With very small theta (< 1e-6) or gamma near 1.0, the denominator can underflow or overflow. Additionally, `(1.0 + theta).powf(1.0 - gamma)` when gamma approaches 1.0 becomes nearly 1, leading to loss of significance.
- Files: `src/ssvi.rs` (line 5-7)
- Impact: Implied volatility calculations become unreliable for very short expiries or extreme parameter combinations. No validation guards against these edge cases.
- Fix approach: (1) Add input validation to ensure theta and gamma are in safe ranges; (2) use log-space computation for phi to avoid underflow: `phi = exp(log(eta) - gamma*log(theta) - (1-gamma)*log(1+theta))`; (3) add numerical gradient checks in tests.

**Grid Search Rho Sweep Not Adaptive:**
- Issue: The calibration algorithm in `src/calibration.rs` (lines 81-106) uses a fixed 20-point grid sweep over rho from -0.95 to 0.95. This is coarse and may miss the optimal rho value if it falls between grid points. The grid is not scaled based on the problem's sensitivity to rho.
- Files: `src/calibration.rs` (lines 79-106)
- Impact: Initial guess for 3D polishing optimization may be suboptimal, leading to longer convergence time or convergence to local minima. Performance degrades for problems where rho is highly sensitive.
- Fix approach: (1) Increase grid resolution; (2) use adaptive coarse-then-fine grid refinement; (3) replace grid search with direct 3D Nelder-Mead from a well-chosen initial point; (4) compute gradient of objective w.r.t. rho to guide search direction.

## Known Bugs

**Brent's Method Returns Midpoint on No Sign Change:**
- Issue: In `src/brent.rs` (lines 18-24), when the function has no sign change in [a,b], the algorithm returns the midpoint as a "best guess" rather than reporting convergence failure clearly. This can mask the underlying problem and produce misleading results.
- Files: `src/brent.rs` (lines 18-24)
- Impact: If the bracketing interval is chosen incorrectly, the solver will return an arbitrary midpoint instead of explicitly failing. Callers cannot distinguish between convergence and failure without checking `.converged` flag.
- Fix approach: This behavior is acceptable for root-finding, but callers must always check `.converged`. Document this clearly in API docs. Consider stricter assertion in calibration when solver claims convergence but fit is poor.

**Unbounded Simplex Expansion in Nelder-Mead:**
- Issue: In `src/nelder_mead.rs` (lines 136-148), the expansion step can push the solution outside the bounds if the expanded point `xe` lands far outside [lb, ub]. The `project()` function clamps it back (line 139), but this can waste iterations and slow convergence.
- Files: `src/nelder_mead.rs` (lines 136-148)
- Impact: Convergence to solutions on the boundary may be unnecessarily slow. The simplex shape can become degenerate.
- Fix approach: Apply box-constraints more intelligently during expansion: limit expansion magnitude to avoid projection clipping, or use constrained expansion formulas.

## Security Considerations

**No Input Validation on Market Data:**
- Issue: Calibration functions `calibrate()` in `src/calibration.rs` accept slices of market total variances without validating that values are positive, finite, or reasonable.
- Files: `src/calibration.rs` (lines 72-142)
- Impact: NaN or infinite values in market data will silently propagate through calculations, producing meaningless calibration results. No early error detection.
- Fix approach: Add precondition checks: ensure all `w_market` values are finite and positive; validate `theta_star > 0` and `k_star` is finite; clip data to reasonable bounds or reject with error.

**No Bounds Checking in Total Variance Calculations:**
- Issue: The `total_variance()` function in `src/ssvi.rs` (line 12-17) computes discriminant `(pk + rho).powi(2) + (1.0 - rho*rho)` without verifying that the square root argument is non-negative. While mathematically guaranteed by construction (discriminant is always ≥ 0 for |rho| ≤ 1), floating-point errors could push it negative.
- Files: `src/ssvi.rs` (lines 12-17)
- Impact: If rho approaches 1.0 or 1+rho is computed with rounding error, the discriminant could be slightly negative, causing `.sqrt()` to return NaN. No guard against this.
- Fix approach: Add assertion or clamp: `disc = disc.max(0.0)` before sqrt. Add tests with rho very close to 1.0.

## Performance Bottlenecks

**Excessive Function Evaluations in Nelder-Mead:**
- Issue: The Nelder-Mead optimizer in `src/nelder_mead.rs` re-computes the objective function for every candidate point without caching. In the calibration pipeline, this means `phi()`, `solve_theta()`, and `total_variance_slice()` are called repeatedly. The `phi()` function itself uses `.powf()` which is slow.
- Files: `src/nelder_mead.rs` (lines 73, 126, 140, 157, 169, 184), `src/calibration.rs` (lines 101, 129)
- Impact: For a 2D (eta, gamma) optimization with typical Nelder-Mead convergence of ~50-200 iterations, combined with 20 rho grid points, the optimizer evaluates the objective 1000-4000 times. Each evaluation calls `solve_theta()` which itself runs Brent's method (~30-50 iterations). Total: ~40,000-200,000 function evaluations. This is slow for interactive use.
- Fix approach: (1) Use analytical gradients to switch to gradient-based optimization (L-BFGS); (2) cache phi computations in a lookup table; (3) parallelize rho grid sweep across threads; (4) use approximate Hessian from optimizer history to reduce iterations.

**Inefficient Phi Computation via Powf:**
- Issue: Computing `eta.powf(gamma)` and `(1.0 + theta).powf(1.0 - gamma)` uses general exponentiation which is slower than specialized functions. For frequently-computed values, this accumulates.
- Files: `src/ssvi.rs` (line 6)
- Impact: Each `phi()` call involves two `powf()` operations, each ~50-100 CPU cycles. Millions of calls in a calibration run add measurable latency.
- Fix approach: (1) Pre-compute phi for a grid of (theta, eta, gamma) values and interpolate; (2) use log-space representation to convert powers to multiplies; (3) profile to determine if this is actually the bottleneck before optimizing.

**Memory Allocation in Simplex Operations:**
- Issue: The Nelder-Mead implementation in `src/nelder_mead.rs` allocates `Vec<Vec<f64>>` for the simplex and clones the entire simplex (line 83) on each iteration to sort it. For 3D optimization, this involves copying 4 vectors of 3 elements each, repeated 50-200 times.
- Files: `src/nelder_mead.rs` (lines 83-86)
- Impact: Unnecessary allocations and copies. Heap fragmentation and cache misses.
- Fix approach: (1) Use stack arrays for small problems (dim <= 10); (2) sort in-place using index vector rather than cloning; (3) use SmallVec for fixed-size simplices.

## Fragile Areas

**Calibration Success Criteria Lack Rigor:**
- Issue: The `calibrate()` function in `src/calibration.rs` (lines 72-142) returns `Some(CalibrationResult)` whenever the final Nelder-Mead optimization completes, regardless of whether the result is actually good. There is no post-hoc validation that the fit is reasonable (e.g., checking max error, parameter bounds, or no-arbitrage saturation).
- Files: `src/calibration.rs` (lines 134-142)
- Impact: A calibration can return "success" with a fit error of 1e-8 (excellent) or 0.1 (terrible) with no way to distinguish. Consumers of the API can unknowingly use bad fits. The `no_arbitrage_satisfied()` check is done during optimization but not on the final result.
- Fix approach: (1) Add optional post-optimization validation step; (2) return additional metadata in `CalibrationResult` (fit error, constraint saturation level); (3) add convenience method `is_good_fit(tolerance)` that checks error magnitude; (4) raise warnings or return `Err` when constraints are saturated.

**Test Assertions Too Strict on Synthetic Data:**
- Issue: Tests in `src/calibration.rs` (lines 206-207, 251-252) assert that recovered SSE is < 1e-20 or 1e-18, which are unrealistically tight tolerances. These tests pass only for synthetic noise-free data. Real market data will never fit this well, making the tests misleading about production performance.
- Files: `src/calibration.rs` (lines 206-207, 251-252)
- Impact: Tests pass locally but code may fail with real data. No confidence that the algorithm handles realistic noise.
- Fix approach: (1) Add noise to synthetic data in tests; (2) use relative error thresholds instead of absolute; (3) add separate stress tests with realistic market scenarios (from `tests/steep_skew.rs`); (4) document acceptable tolerance ranges for real data.

**Plot Generation Assumes Data Properties:**
- Issue: In `src/bin/report.rs` (lines 116-125), the plot generation uses `.first()` and `.last()` with fallback values, but assumes k_slice is sorted and non-empty. If market data has outliers or gaps, the plot axis scaling (lines 124-125) uses `fold()` to find min/max, which is inefficient and could overflow with extreme values.
- Files: `src/bin/report.rs` (lines 116-125)
- Impact: Edge cases (unsorted data, NaN values, empty slices) produce misleading visualizations. Plot axes may be scaled incorrectly if outliers are present.
- Fix approach: (1) Validate input data before plotting; (2) use quantile-based axis scaling (e.g., 5th-95th percentile) instead of min-max to handle outliers; (3) use iterator methods more efficiently (`.min_by`, `.max_by`).

**Hard-Coded Scenario Parameters in Report:**
- Issue: The report generation in `src/bin/report.rs` (lines 269-275) has hard-coded test scenarios with fixed T, slope, and ATM vol values. To test different market conditions, users must edit and recompile the binary.
- Files: `src/bin/report.rs` (lines 269-277)
- Impact: Report is not reusable for different market regimes. Cannot be integrated into automated testing or used as a library.
- Fix approach: (1) Move scenario parameters to command-line arguments or config file; (2) make report generation a public library function in lib.rs; (3) support reading market data from CSV or JSON.

## Scaling Limits

**Rho Grid Sweep Becomes Slow with High-Dimensional Problems:**
- Issue: The current 20-point rho grid (line 80 in `src/calibration.rs`) scales linearly with the number of outer iterations. If the algorithm is extended to calibrate multiple slices or parameters, the cost multiplies.
- Files: `src/calibration.rs` (lines 79-106)
- Impact: Calibrating an entire volatility surface (20+ time slices) would require 20*20 = 400 calibrations, each doing 20-iteration rho sweep, resulting in thousands of objective evaluations.
- Scaling path: (1) Make rho grid resolution configurable; (2) use faster grid search (e.g., ternary search) instead of linear scan; (3) parallelize over rho values; (4) for surface calibration, use previous slice's rho as warm-start.

**Simplex Dimension Grows Cubically with Problem Size:**
- Issue: Nelder-Mead simplex has n+1 vertices where n is dimension. For 3D calibration (eta, gamma, rho), the simplex is 4 vertices. Extending to calibrate more parameters (e.g., theta directly, or multiple SSVI components) rapidly increases cost.
- Files: `src/nelder_mead.rs` (lines 53-71)
- Impact: 10D optimization requires a 11-vertex simplex with 11 function evaluations per iteration. Convergence becomes exponentially slower.
- Scaling path: (1) Use gradient-based methods (L-BFGS) instead of simplex for higher dimensions; (2) use coordinate descent or alternating optimization if structure allows; (3) reduce problem dimension via constraints or parameterization.

## Dependencies at Risk

**Plotters Dependency for Simple Plotting:**
- Risk: The crate depends on `plotters = "0.3"` for SVG generation. Plotters is a heavy dependency (~200KB) with many transitive dependencies. SVG generation is relatively simple and could be done with minimal code.
- Files: `Cargo.toml` (line 6), `src/bin/report.rs`
- Impact: Increased compilation time, larger binary, potential security vulnerabilities in upstream deps. If plotters is abandoned or has breaking changes, the reporting feature breaks.
- Migration plan: (1) For MVP, use hand-written SVG templates with data interpolation (10-20 lines of code); (2) for production, evaluate `plotly.rs` or `gnuplot` integration; (3) or keep plotters but use `optional` feature flag.

## Missing Critical Features

**No Error Reporting Mechanism:**
- Problem: Calibration failures return `None` with no indication of what went wrong. Was it: bracket failure, non-convergence, invalid input, or numerical instability?
- Blocks: Debugging production failures; building robust user-facing error messages; automated failure triage.
- Fix: Replace `Option<CalibrationResult>` with `Result<CalibrationResult, CalibrationError>` enum. Document each error variant.

**No Confidence Intervals or Uncertainty Estimates:**
- Problem: The calibration returns point estimates (eta, gamma, rho) but no measure of how uncertain these estimates are. For a poorly-constrained problem or noisy data, the estimates could have high variance.
- Blocks: Risk quantification; parameter sensitivity analysis; intelligent fallback strategies.
- Fix: Compute Hessian at optimum to estimate parameter covariance. Return confidence regions with results.

**No Warm-Start or Incremental Calibration:**
- Problem: Each calibration starts from scratch with default initial simplex. If calibrating time-adjacent market slices, parameters should change smoothly, but the algorithm doesn't exploit this.
- Blocks: Rapid re-calibration; real-time volatility surface updates.
- Fix: Accept optional previous-fit as warm-start. Initialize simplex around previous solution.

## Test Coverage Gaps

**No Integration Tests with Real Market Data:**
- What's not tested: How the algorithm performs on actual equity options data with noise, bid-ask spreads, and outliers. Only synthetic noise-free scenarios are tested.
- Files: `src/calibration.rs` (test module), `tests/steep_skew.rs`
- Risk: Code may fail silently or produce poor fits on production data. Test suite provides false confidence.
- Priority: High

**Limited Edge Case Coverage for Input Validation:**
- What's not tested: Behavior with NaN/Inf market data, zero variance values, empty slices, very large parameter values, degenerate input (all k_slice values identical).
- Files: `src/calibration.rs` (lines 72-142), `src/ssvi.rs` (lines 12-17)
- Risk: Undefined behavior or crashes on malformed input.
- Priority: High

**No Stress Tests for Extreme Parameters:**
- What's not tested: Calibration with very small T (< 0.001), very large skew (slope > 2.0), extreme ATM vols (0.01 or 2.0), rho very close to ±1.
- Files: `src/calibration.rs`, `src/ssvi.rs`
- Risk: Algorithm may silently produce nonsense results (NaN, Inf, negative variance) that only appear in downstream analytics.
- Priority: Medium

**No Benchmark Coverage for Performance Regression:**
- What's not tested: Total runtime of calibration; sensitivity to problem size; memory usage. Benches exist in `benches/calibration.rs` but are minimal.
- Files: `benches/calibration.rs`
- Risk: Performance regressions are not caught in CI. Algorithm changes may introduce O(n²) overhead undetected.
- Priority: Medium

---

*Concerns audit: 2026-03-05*
