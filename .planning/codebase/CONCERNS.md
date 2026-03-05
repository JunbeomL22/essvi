# Codebase Concerns

**Analysis Date:** 2026-03-05

## Tech Debt

**Scaffold-only codebase - no actual implementation:**
- Issue: The project is a brand-new Rust library (`essvi`) intended to implement SSVI (Surface SVI) calibration with a bounded Nelder-Mead optimizer. Currently `src/lib.rs` contains only the default `cargo new --lib` scaffold (a trivial `add` function and one test). None of the domain logic described in `documents/guideline.md` has been implemented.
- Files: `src/lib.rs`
- Impact: The project is non-functional. No SSVI model, no optimizer, no calibration pipeline exists yet.
- Fix approach: Implement the modules described in the guideline: Nelder-Mead optimizer, SSVI surface model, theta solver (Brent root-finding), and the calibration orchestration. Follow the architecture outlined in `documents/guideline.md`.

**No module structure:**
- Issue: The project has a single `src/lib.rs` with no module hierarchy. The guideline describes at least 3-4 distinct components (optimizer, SSVI model, root solver, calibration) that will need separation.
- Files: `src/lib.rs`
- Impact: As implementation grows, a flat single-file structure will become unmaintainable.
- Fix approach: Create a module structure early, e.g., `src/optimizer.rs` (Nelder-Mead), `src/ssvi.rs` (SSVI model and phi function), `src/solver.rs` (Brent root-finding for theta), and `src/calibration.rs` (orchestration). Re-export public API from `src/lib.rs`.

## Known Bugs

No bugs to report - the codebase has no domain logic implemented yet.

## Security Considerations

**No sensitive data handling detected:**
- Risk: Low. This is a numerical computation library with no network, file I/O, or user input processing.
- Files: `src/lib.rs`
- Current mitigation: Not applicable.
- Recommendations: None at this stage.

## Performance Bottlenecks

**Potential NaN/Infinity propagation in numerical code (future risk):**
- Problem: The SSVI model involves division, square roots, and power functions that can produce NaN or Infinity for edge-case parameter combinations (e.g., theta near zero, gamma near boundary values).
- Files: Not yet implemented, but will affect future `src/ssvi.rs`, `src/solver.rs`
- Cause: Floating-point edge cases in `theta^gamma`, `(1+theta)^(1-gamma)`, and division in `varphi(theta)`.
- Improvement path: Add explicit guards for degenerate parameter values. Return a large penalty value from the objective function when NaN/Infinity is detected, rather than propagating it through the optimizer.

**Nelder-Mead convergence for ill-conditioned surfaces (future risk):**
- Problem: The guideline's Nelder-Mead implementation uses `partial_cmp(...).unwrap()` for sorting, which will panic on NaN function values. Additionally, the projection-based bound handling can cause the simplex to collapse when the optimum lies exactly on a boundary.
- Files: Algorithm described in `documents/guideline.md` (Section 4.3, lines 262-265)
- Cause: `unwrap()` on `partial_cmp` with potential NaN values; simplex degeneracy near bounds.
- Improvement path: Use `partial_cmp(...).unwrap_or(std::cmp::Ordering::Greater)` to handle NaN gracefully. Add simplex degeneracy detection and restart logic.

## Fragile Areas

**Implicit theta solver (future risk):**
- Files: Not yet implemented; described in `documents/guideline.md` Section 2.3
- Why fragile: The fixed-point iteration `theta_{n+1} = g(theta_n)` for solving the implicit equation may not converge for all parameter combinations. The Brent method requires a valid bracketing interval `[a, b]` where `h(a)` and `h(b)` have opposite signs, which is not guaranteed for arbitrary `(eta, gamma, rho)`.
- Safe modification: Always validate that the root-finding method converged before using the resulting theta. Return an `Option<f64>` or `Result<f64, Error>` from the solver, not a bare `f64`.
- Test coverage: No tests exist yet. Need tests for: convergence with known-good parameters, graceful failure for degenerate parameters, boundary cases where theta approaches zero.

## Scaling Limits

**Not applicable at current stage.** The problem is inherently small-scale (3 optimization variables, per-slice calibration). The guideline notes this explicitly.

## Dependencies at Risk

**Zero external dependencies:**
- Risk: None. `Cargo.toml` has no `[dependencies]` entries. The project uses only the Rust standard library.
- Impact: Positive - no supply chain risk. However, this means all numerical routines (Brent root-finding, Nelder-Mead) must be implemented and tested from scratch.
- Migration plan: Not applicable.

**Rust edition 2024:**
- Risk: `Cargo.toml` specifies `edition = "2024"`. This is a very recent Rust edition and may have limited toolchain support on older Rust installations.
- Files: `Cargo.toml`
- Impact: Build failures if the Rust toolchain is not sufficiently up-to-date.
- Migration plan: Ensure the development environment uses Rust 1.85+ (the first release supporting edition 2024). Alternatively, fall back to `edition = "2021"` if compatibility is needed.

## Missing Critical Features

**Entire domain implementation is missing:**
- Problem: The guideline in `documents/guideline.md` describes a complete SSVI calibration system. None of it exists in code yet.
- Blocks: The library cannot be used for any purpose until the following are implemented:
  1. SSVI surface model (`w(k, theta)` and `varphi(theta)`)
  2. Brent root-finding for theta (implicit equation solver)
  3. Bounded Nelder-Mead optimizer
  4. Calibration orchestration (objective function, parameter bounds)

**No error types defined:**
- Problem: No custom error types or result types exist for handling calibration failures (non-convergence, invalid parameters, NaN results).
- Blocks: Robust API design. Callers will have no way to distinguish between different failure modes.

**No public API design:**
- Problem: No traits, public structs, or function signatures exist for the calibration API. The guideline provides implementation details but no API design.
- Blocks: Integration into downstream applications.

## Test Coverage Gaps

**No meaningful tests:**
- What's not tested: Everything. The only test is the scaffold `it_works` test for the placeholder `add` function in `src/lib.rs`.
- Files: `src/lib.rs`
- Risk: Once implementation begins, lack of tests for numerical routines (optimizer convergence, root-finding accuracy, SSVI model correctness) could lead to subtle numerical bugs that are hard to detect.
- Priority: High. The guideline (Section 4.5) provides expected test results (Rosenbrock 2D, bounded optimization, SSVI-like 3D) that should be implemented as regression tests from the start.

---

*Concerns audit: 2026-03-05*
