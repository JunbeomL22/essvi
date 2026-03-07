# Codebase Structure

**Analysis Date:** 2026-03-07

## Directory Layout

```
essvi/
├── src/
│   ├── lib.rs              # Crate root: re-exports all public modules
│   ├── ssvi.rs             # SSVI model formulas (phi, total_variance, no_arbitrage)
│   ├── calibration.rs      # Calibration pipeline (solve_theta, calibrate, calendar penalty)
│   ├── nelder_mead.rs      # Bounded Nelder-Mead optimizer
│   ├── brent.rs            # Brent's method root finder
│   └── bin/
│       ├── report.rs       # Parameter grid sweep report generator
│       ├── fit_real.rs     # Per-slice real-world-like data fitting
│       └── fit_real_surface.rs  # Surface fit with calendar arbitrage penalty
├── tests/
│   └── steep_skew.rs       # Integration stress tests for steep skew regimes
├── benches/
│   └── calibration.rs      # Criterion benchmarks for all calibration paths
├── documents/
│   ├── plots/              # Generated SVG fit plots (~80 files)
│   ├── fit_quality_report.md   # Grid sweep report output
│   ├── real-world-fit.md       # Per-slice fit report output
│   ├── real-world-surface-fit.md  # Surface fit report output
│   ├── remedy.md           # Calendar arbitrage remedy notes
│   ├── theta-calc.md       # Theta calculation derivation
│   ├── guideline.md        # Project guidelines
│   ├── real-data.png       # Reference market data screenshot
│   ├── svi.pdf             # SVI reference paper
│   └── robust-calibration.pdf  # Calibration reference paper
├── .planning/
│   ├── PROJECT.md          # Project definition and milestones
│   └── codebase/           # Architecture/quality analysis documents (this directory)
├── Cargo.toml              # Package manifest (edition 2024)
├── Cargo.lock              # Dependency lockfile
├── .gitignore              # Ignores /target only
└── README.md               # Minimal placeholder
```

## Directory Purposes

**`src/`:**
- Purpose: All library source code (4 modules) and binary targets
- Contains: Rust source files (`.rs`)
- Key files: `lib.rs` (crate root), `calibration.rs` (largest module, 388 lines), `ssvi.rs` (core model)

**`src/bin/`:**
- Purpose: Standalone binary executables for report generation and demonstration
- Contains: Three binaries that import the library and produce markdown reports + SVG plots
- Key files: `fit_real.rs` (per-slice fitting), `fit_real_surface.rs` (surface fitting with calendar penalty), `report.rs` (parameter grid analysis)

**`tests/`:**
- Purpose: Integration tests that exercise the full calibration pipeline
- Contains: Stress tests for extreme parameter regimes (steep skew, near-zero expiry)
- Key files: `steep_skew.rs` (tests calibration across T=[1.0, 0.1, 0.01, 0.001])

**`benches/`:**
- Purpose: Performance benchmarks using Criterion
- Contains: Benchmarks for `solve_theta`, `total_variance_slice`, per-slice calibration, and full 12-slice surface calibration
- Key files: `calibration.rs`

**`documents/`:**
- Purpose: Reference materials, generated reports, and fit plots
- Contains: PDF reference papers, markdown reports, PNG reference data, SVG plots
- Key files: `guideline.md`, `fit_quality_report.md`, `real-world-fit.md`, `real-world-surface-fit.md`

**`documents/plots/`:**
- Purpose: Generated SVG visualizations of fit results
- Contains: ~80 SVG files showing market data vs SSVI fit curves
- Generated: Yes (by running binaries)
- Committed: Yes

**`.planning/`:**
- Purpose: Project planning and analysis documents
- Contains: `PROJECT.md` (project definition, milestones, decisions), `codebase/` subdirectory for architecture docs
- Generated: No (manually maintained)
- Committed: Yes

## Key File Locations

**Entry Points:**
- `src/lib.rs`: Crate root, declares public modules `brent`, `calibration`, `nelder_mead`, `ssvi`
- `src/bin/report.rs`: Parameter grid sweep binary (run: `cargo run --bin report`)
- `src/bin/fit_real.rs`: Per-slice fitting binary (run: `cargo run --bin fit_real`)
- `src/bin/fit_real_surface.rs`: Surface fitting binary (run: `cargo run --bin fit_real_surface`)

**Configuration:**
- `Cargo.toml`: Package manifest -- edition 2024, single dependency `plotters 0.3`, dev-dependency `criterion 0.5`
- No runtime configuration files; all parameters are hardcoded in source

**Core Logic:**
- `src/ssvi.rs`: SSVI model -- `phi()`, `total_variance()`, `total_variance_slice()`, `no_arbitrage_satisfied()`
- `src/calibration.rs`: Calibration pipeline -- `solve_theta()`, `calibrate()`, `calibrate_with_calendar_penalty()`, plus data structs `CalibrationInput`, `CalibrationResult`, `PrevSlice`
- `src/nelder_mead.rs`: Bounded Nelder-Mead optimizer -- `nelder_mead_bounded()`, `NelderMeadConfig`, `NelderMeadResult`
- `src/brent.rs`: Brent root finder -- `brent()`, `BrentResult`

**Testing:**
- `src/ssvi.rs` (inline `#[cfg(test)]` mod): Unit tests for phi, ATM total variance, no-arb check
- `src/calibration.rs` (inline `#[cfg(test)]` mod): Unit tests for solve_theta, calibrate round-trip, no-arb enforcement
- `src/nelder_mead.rs` (inline `#[cfg(test)]` mod): Rosenbrock 2D test, boundary solution test
- `src/brent.rs` (inline `#[cfg(test)]` mod): sqrt(2) root finding, no-sign-change handling
- `tests/steep_skew.rs`: Integration stress test across extreme parameter regimes
- `benches/calibration.rs`: Criterion benchmarks for performance regression tracking

## Naming Conventions

**Files:**
- Library modules: `snake_case.rs` (e.g., `nelder_mead.rs`, `calibration.rs`)
- Binary targets: `snake_case.rs` (e.g., `fit_real.rs`, `fit_real_surface.rs`)
- Test files: `snake_case.rs` matching the feature under test (e.g., `steep_skew.rs`)

**Directories:**
- All lowercase: `src/`, `tests/`, `benches/`, `documents/`, `documents/plots/`
- Standard Rust project layout (no custom directory patterns)

**Functions:**
- Public API: `snake_case` (e.g., `total_variance()`, `solve_theta()`, `calibrate()`, `nelder_mead_bounded()`)
- Internal helpers: `snake_case` (e.g., `weighted_squared_error()`, `calendar_penalty()`, `project()`)

**Types:**
- Structs: `PascalCase` (e.g., `CalibrationInput`, `NelderMeadConfig`, `CalibrationResult`, `PrevSlice`, `BrentResult`)
- No enums, traits, or type aliases in the current codebase

**Variables:**
- Mathematical variables preserved from academic notation: `eta`, `gamma`, `rho`, `theta`, `phi`, `k`, `w`
- Descriptive names for composite values: `theta_star`, `k_star`, `w_market`, `k_slice`, `no_arb_usage`
- Loop counters and temporaries: `i`, `j`, `n`, `t`, `s`, `pk`, `dk`

## Where to Add New Code

**New Model (e.g., eSSVI):**
- Create: `src/essvi.rs` (or `src/essvi_model.rs`) as a new module
- Register in: `src/lib.rs` -- add `pub mod essvi_model;`
- Follow pattern of: `src/ssvi.rs` -- pure functions, `#[inline]` on hot-path evaluators, `#[cfg(test)]` module at bottom

**New Calibration Pipeline (e.g., eSSVI calibration):**
- Create: `src/essvi_calibration.rs` (separate from existing `calibration.rs`)
- Follow pattern of: `src/calibration.rs` -- `CalibrationInput`-like struct, `Option`-returning calibrate function, reuse `nelder_mead_bounded()`
- Register in: `src/lib.rs`

**New Binary (e.g., comparative report):**
- Create: `src/bin/compare.rs` (or similar descriptive name)
- Follow pattern of: `src/bin/report.rs` -- define data structs, run calibration, generate SVG plots via `plotters`, write markdown report
- No Cargo.toml changes needed (Cargo auto-discovers `src/bin/*.rs`)

**New Integration Test:**
- Create: `tests/new_test_name.rs`
- Follow pattern of: `tests/steep_skew.rs` -- import `essvi::calibration::*` and `essvi::ssvi`, use `#[test]` functions
- No Cargo.toml changes needed (Cargo auto-discovers `tests/*.rs`)

**New Benchmark:**
- Add to: `benches/calibration.rs` -- define a new `bench_*` function, add to `criterion_group!`
- Follow pattern of: existing bench functions that use `black_box()` for inputs

**New Numerical Solver:**
- Create: `src/solver_name.rs`
- Register in: `src/lib.rs`
- Follow pattern of: `src/brent.rs` or `src/nelder_mead.rs` -- generic function taking a closure, config struct with `Default`, result struct with convergence flag

**Shared Utilities:**
- Currently no dedicated utilities module exists
- Small helpers live in the module that uses them (e.g., `weighted_squared_error()` in `calibration.rs`, `project()` in `nelder_mead.rs`)
- If cross-module utilities are needed, create `src/utils.rs` and register in `src/lib.rs`

## Special Directories

**`target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes
- Committed: No (in `.gitignore`)

**`documents/plots/`:**
- Purpose: SVG fit visualizations generated by binaries
- Generated: Yes (by `cargo run --bin fit_real`, `cargo run --bin fit_real_surface`, `cargo run --bin report`)
- Committed: Yes (tracked in git for documentation purposes)

**`.planning/codebase/`:**
- Purpose: Architecture and quality analysis documents consumed by planning tools
- Generated: Semi-automated (written by analysis agents)
- Committed: Yes

---

*Structure analysis: 2026-03-07*
