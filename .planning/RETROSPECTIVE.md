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

## Milestone: v1.1 — Pricing Primitives

**Shipped:** 2026-03-07
**Phases:** 3 | **Plans:** 3 | **Sessions:** 1

### What Was Built
- Math foundations: fdlibm-based erf/erfc/erfcx, standard normal (PDF/CDF/inverse), high-precision normal CDF with asymptotic expansion, numerical constants
- Black-76 pricing: undiscounted/discounted pricing, individual and combined greeks, PricingError type
- Implied volatility solver: normalised Black call/vega, bisection initial guess + Halley refinement, rational cubic interpolation

### What Worked
- YOLO mode completed all 3 phases in a single subagent context — no per-phase Task() spawning needed
- Single-plan-per-phase pattern continued to work well for well-scoped numerical code
- The agent correctly identified and fixed 4 bugs during implementation (threshold inversion, tolerance, missing factor, wrong formula)
- 111 tests (91 integration + 20 doc-tests) provide strong coverage

### What Was Inefficient
- YOLO subagent again skipped GSD phase directory creation (no PLAN.md/VERIFICATION.md/SUMMARY.md) — milestone stats reconstructed from git for second time
- All 3 phases ran in a single subagent context instead of per-phase Task() spawning as designed — works but loses per-phase verification artifacts
- Roadmap `analyze` tool reported `next_phase: 6` even though all checkboxes were ticked — tooling keys on disk artifacts, not roadmap state

### Patterns Established
- Numerical Rust modules benefit from comprehensive doc-tests that serve as both documentation and regression tests
- `i32 q` convention (+1/-1 for call/put) is simpler than enum when matching upstream algorithm APIs
- Error types should cover the semantic domain (AboveMaximum, BelowIntrinsic, InvalidInput) rather than be generic

### Key Lessons
1. YOLO subagents completing all phases in one context is efficient but loses GSD verification artifacts — consider whether this tradeoff is acceptable
2. For numerical code, the agent can self-correct bugs during implementation when tests are written alongside the code
3. The `roadmap analyze` tool's completion detection should use roadmap checkbox state, not just disk artifacts

### Cost Observations
- Model mix: 100% opus (quality profile)
- Sessions: 1 (full YOLO run)
- Notable: 3 phases completed in ~47 minutes of agent time; 2,379 new LOC Rust

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | 1 | 5 | Initial milestone, YOLO mode, single-plan phases |
| v1.1 | 1 | 3 | Numerical code, all phases in single subagent context |

### Cumulative Quality

| Milestone | Tests | Coverage | Zero-Dep Additions |
|-----------|-------|----------|-------------------|
| v1.0 | 14 | Unit + integration | CalibError, CalibrationConfig |
| v1.1 | 111 | Unit + integration + doc-tests | math/, pricing/ modules, PricingError |

### Top Lessons (Verified Across Milestones)

1. Single-plan phases with YOLO mode is the most efficient pattern for well-scoped work — verified across both v1.0 (5 phases) and v1.1 (3 phases)
2. YOLO subagents consistently skip GSD phase directory creation — milestone stats must be reconstructed from git (observed in both v1.0 and v1.1)
3. Pure Rust zero-dependency modules are straightforward to implement — no external dependency coordination needed
