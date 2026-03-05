# Codebase Structure

**Analysis Date:** 2026-03-05

## Directory Layout

```
essvi/
├── Cargo.toml              # Package manifest (Rust edition 2024, dependencies)
├── Cargo.lock              # Dependency lock file
├── README.md               # Minimal project readme
├── .gitignore              # Git ignore patterns
├── .planning/
│   ├── PROJECT.md          # Project documentation
│   └── codebase/           # Codebase analysis (this directory)
├── src/
│   ├── lib.rs              # Library root (module declarations)
│   ├── ssvi.rs             # SSVI volatility model mathematics
│   ├── brent.rs            # Brent's root-finding algorithm
│   ├── nelder_mead.rs      # Bounded Nelder-Mead optimizer
│   ├── calibration.rs      # SSVI parameter calibration
│   └── bin/
│       └── report.rs       # Diagnostic report generator (binary)
├── tests/
│   └── steep_skew.rs       # Stress test: near-zero expiry + steep skew
├── benches/
│   └── calibration.rs      # Performance benchmarks (criterion)
├── documents/
│   ├── fit_quality_report.md    # Generated fit quality report
│   ├── guideline.md             # Usage guidelines
│   └── plots/                    # Generated SVG plots
└── target/                 # Build artifacts (not committed)
```

## Directory Purposes

**`src/`:**
- Purpose: Library source code (modules, core logic)
- Contains: Rust source files (.rs) implementing numerical algorithms and volatility model
- Key files: `lib.rs` (module root), `ssvi.rs`, `calibration.rs`, `brent.rs`, `nelder_mead.rs`

**`src/bin/`:**
- Purpose: Standalone executable programs
- Contains: Command-line binaries using the library
- Key files: `report.rs` (generates diagnostic report and plots)

**`tests/`:**
- Purpose: Integration tests
- Contains: Test files executed with `cargo test`
- Key files: `steep_skew.rs` (stress tests for extreme parameter ranges)

**`benches/`:**
- Purpose: Performance benchmarks
- Contains: Criterion benchmark definitions
- Key files: `calibration.rs` (measures calibration speed, theta solving, batch variance computation)

**`documents/`:**
- Purpose: Generated documentation and plots
- Contains: Markdown reports and SVG visualizations
- Auto-generated: `fit_quality_report.md`, `plots/*.svg`
- Pre-existing: `guideline.md`

**`.planning/`:**
- Purpose: GSD planning and codebase analysis
- Contains: Project documentation and codebase mapping
- Key files: `PROJECT.md`, `codebase/*.md`

## Key File Locations

**Entry Points:**
- `src/lib.rs`: Library entry — declares and re-exports modules (brent, calibration, nelder_mead, ssvi)
- `src/bin/report.rs`: Binary entry — generates full diagnostic report with plots
- `tests/steep_skew.rs`: Test entry — integration tests with synthetic extreme scenarios
- `benches/calibration.rs`: Benchmark entry — performance measurements

**Configuration:**
- `Cargo.toml`: Rust package configuration (edition, dependencies: plotters, criterion)
- `Cargo.lock`: Locked dependency versions

**Core Logic:**
- `src/ssvi.rs`: SSVI mathematical model (phi function, total variance, no-arbitrage check)
- `src/calibration.rs`: Parameter estimation (solve_theta implicit solver, grid-sweep + polish)
- `src/brent.rs`: Root-finding algorithm
- `src/nelder_mead.rs`: Bounded optimization algorithm

**Testing:**
- `tests/steep_skew.rs`: Stress test validating calibration under extreme conditions
- `benches/calibration.rs`: Performance benchmarks

**Output:**
- `documents/fit_quality_report.md`: Auto-generated report (parametric grid results, constraint saturation)
- `documents/plots/fit_*.svg`: Auto-generated fit plots (market vs model)

## Naming Conventions

**Files:**
- Library modules: lowercase with underscores — `ssvi.rs`, `brent.rs`, `nelder_mead.rs`, `calibration.rs`
- Binary: `bin/report.rs`
- Tests: descriptive snake_case — `steep_skew.rs`
- Benchmarks: descriptive snake_case — `calibration.rs`

**Directories:**
- Standard Rust: `src/`, `tests/`, `benches/`, `target/`
- Project-specific: `documents/` (reports, plots), `.planning/` (planning docs)
- Generated: `documents/plots/` (SVG output), `target/debug/`, `target/release/`

**Structs & Types:**
- Configuration: `NelderMeadConfig`, `CalibrationInput`, `CalibrationResult`, `BrentResult`, `NelderMeadResult`
- Case: PascalCase (standard Rust convention)

**Functions:**
- Case: snake_case (standard Rust convention)
- Examples: `phi()`, `total_variance()`, `solve_theta()`, `calibrate()`, `brent()`, `nelder_mead_bounded()`

**Modules:**
- Case: lowercase (standard Rust convention)
- Public re-exports in `lib.rs`: `pub mod brent;`, `pub mod calibration;`, etc.

## Where to Add New Code

**New Feature (e.g., alternative volatility model):**
- Primary code: `src/new_model.rs` (new module for model implementation)
- Re-export: Add `pub mod new_model;` to `src/lib.rs`
- Tests: Add test module within `src/new_model.rs` under `#[cfg(test)]`
- Integration tests: Create `tests/test_new_model.rs` for cross-module validation

**New Algorithm/Solver:**
- Location: `src/algorithm_name.rs` (e.g., `src/gradient_descent.rs`)
- Pattern: Follow Brent/Nelder-Mead structure: generic function, result struct with convergence info
- Re-export: Add to `lib.rs`
- Tests: Inline tests in same file (test against known problems: Rosenbrock, sqrt, etc.)

**New Optimization Objective (e.g., different calibration approach):**
- Primary code: New function in `src/calibration.rs` (e.g., `calibrate_tikhonov()`)
- Input: Extend or create companion struct (similar to `CalibrationInput`)
- Output: Return `Option<CalibrationResult>` or new result type
- Tests: Add test cases in `#[cfg(test)]` block within `calibration.rs`

**New Binary/CLI Tool:**
- Location: `src/bin/tool_name.rs`
- Dependencies: Import from library via `use essvi::*;`
- Pattern: Follow `report.rs` structure (scenario generation, computation, output)

**Unit Tests:**
- Location: Inline in same module, under `#[cfg(test)] mod tests { ... }`
- Naming: Descriptive function names, `#[test]` attribute
- Examples: `solve_theta_basic()`, `calibrate_recovers_parameters()`, `no_arb_enforced()`

**Integration Tests:**
- Location: `tests/*.rs`
- Naming: Descriptive file names matching test subject
- Pattern: Test cross-module interactions (e.g., full calibration pipeline)

**Utilities/Helpers:**
- Shared math utilities: Add as functions to relevant module (e.g., `ssvi.rs`)
- Shared output/formatting: Create `src/output.rs` if needed
- General helpers: Keep in existing modules, avoid separate util file unless >100 lines

## Special Directories

**`target/`:**
- Purpose: Build artifacts (Rust compiled output)
- Generated: Yes (created by cargo)
- Committed: No (in `.gitignore`)
- Contents: `debug/` and `release/` with compiled binaries, dependencies, build metadata

**`documents/plots/`:**
- Purpose: Generated SVG plot outputs
- Generated: Yes (created by `cargo run --bin report`)
- Committed: Partially (many plots untracked)
- Contents: `fit_*.svg` (fit quality plots), `heatmap_*.svg` (constraint analysis heatmaps)

**`.planning/codebase/`:**
- Purpose: GSD codebase analysis documents
- Generated: By GSD map-codebase command
- Committed: Yes
- Contents: `ARCHITECTURE.md`, `STRUCTURE.md` (this file), and related analysis documents

## Code Organization Patterns

**Module Structure (src/lib.rs):**
```rust
pub mod brent;        // Root finding
pub mod calibration;  // Parameter estimation
pub mod nelder_mead;  // Optimization
pub mod ssvi;         // Volatility model
```

**Module Internal Structure (example: src/ssvi.rs):**
```rust
// 1. Imports (if any)
use crate::other_module;

// 2. Public API (functions, types)
pub fn phi(theta: f64, eta: f64, gamma: f64) -> f64 { ... }
pub fn total_variance(...) -> f64 { ... }
pub fn no_arbitrage_satisfied(...) -> bool { ... }

// 3. Private helpers (if any)
fn helper_fn(...) { ... }

// 4. Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() { ... }
}
```

**Binary Structure (example: src/bin/report.rs):**
```rust
// 1. Imports
use essvi::*;
use std::fs;

// 2. Helper structs (Scenario, FitResult, etc.)
struct Scenario { ... }
struct FitResult { ... }

// 3. Core functions (run_scenario, plot_fit, etc.)
fn run_scenario(s: Scenario) -> Option<FitResult> { ... }
fn plot_fit(result: &FitResult, path: &str) -> Result<(), Box<dyn std::error::Error>> { ... }

// 4. Main entry
fn main() { ... }
```

---

*Structure analysis: 2026-03-05*
