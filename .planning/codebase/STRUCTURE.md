# Codebase Structure

**Analysis Date:** 2026-03-05

## Directory Layout

```
essvi/
├── src/                # Rust source code
│   └── lib.rs          # Library crate root (scaffold only)
├── documents/          # Design documents and mathematical guidelines
│   └── guideline.md    # SSVI calibration algorithm design (Korean)
├── Cargo.toml          # Package manifest (edition 2024, no dependencies)
├── Cargo.lock          # Dependency lockfile (no external deps)
├── .gitignore          # Ignores /target
└── target/             # Build artifacts (git-ignored)
```

## Directory Purposes

**`src/`:**
- Purpose: All Rust library source code
- Contains: Currently only the crate root `lib.rs` with a placeholder function
- Key files: `src/lib.rs`

**`documents/`:**
- Purpose: Mathematical and algorithmic design documentation
- Contains: Detailed SSVI calibration methodology, optimization approach analysis, and reference Rust code snippets
- Key files: `documents/guideline.md`

**`target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes
- Committed: No (git-ignored)

## Key File Locations

**Entry Points:**
- `src/lib.rs`: Library crate root. Currently a scaffold; will become the module re-export hub.

**Configuration:**
- `Cargo.toml`: Package manifest. Edition 2024, no external dependencies.

**Core Logic:**
- `src/lib.rs`: Will contain or re-export optimizer, model, and calibration code.

**Design Reference:**
- `documents/guideline.md`: Complete algorithmic specification with reference implementations in Rust. Written in Korean. Contains the Nelder-Mead optimizer code, SSVI formulas, and usage examples.

**Testing:**
- `src/lib.rs`: Contains inline `#[cfg(test)] mod tests` block with a single placeholder test.

## Naming Conventions

**Files:**
- Snake_case for Rust source files: `lib.rs`
- Lowercase for document files: `guideline.md`

**Directories:**
- Lowercase singular: `src/`, `documents/`

**Rust Items (from design document patterns):**
- Structs: PascalCase (`NelderMeadConfig`, `NelderMeadResult`)
- Functions: snake_case (`nelder_mead_bounded`, `project`)
- Constants/parameters: snake_case (`max_iter`, `tol_f`)

## Where to Add New Code

**New Module (e.g., optimizer, SSVI model, calibration):**
- Create file at `src/<module_name>.rs`
- Add `pub mod <module_name>;` to `src/lib.rs`
- Re-export key types: `pub use <module_name>::{Type1, Type2};`

**New Feature (optimizer implementation):**
- Primary code: `src/optimizer.rs` (or `src/nelder_mead.rs`)
- Structs to define: `NelderMeadConfig`, `NelderMeadResult`
- Main function: `pub fn nelder_mead_bounded<F: Fn(&[f64]) -> f64>(...)`
- Tests: Inline `#[cfg(test)] mod tests` at bottom of same file

**SSVI Model Code:**
- Implementation: `src/ssvi.rs` or `src/model.rs`
- Functions: `ssvi_total_variance(k, theta, eta, gamma, rho)`, `phi(theta, eta, gamma)`

**Calibration Orchestrator:**
- Implementation: `src/calibration.rs`
- Functions: `calibrate(market_data, initial_guess, config)`, `solve_theta(eta, gamma, rho, theta_star, k_star)`

**Theta Root Solver:**
- Implementation: `src/brent.rs` or include in `src/calibration.rs`
- Function: `brent_root(f, a, b, tol)` or fixed-point iteration

**Tests:**
- Place inline with code using `#[cfg(test)] mod tests { ... }` blocks
- For integration tests, create `tests/` directory at project root

**New Design Documents:**
- Place in `documents/` directory

## Special Directories

**`target/`:**
- Purpose: Cargo build output (debug/release binaries, intermediate artifacts)
- Generated: Yes (by `cargo build`)
- Committed: No

**`.planning/`:**
- Purpose: GSD planning and codebase analysis documents
- Generated: Yes (by tooling)
- Committed: Project decision

---

*Structure analysis: 2026-03-05*
