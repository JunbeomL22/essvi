# Technology Stack

**Analysis Date:** 2026-03-08

## Languages

**Primary:**
- Rust (Edition 2024) - All library code, binaries, tests, and benchmarks

**Secondary:**
- Python 3 - Data acquisition script (`scripts/fetch_options.py`)

## Runtime

**Environment:**
- Rust stable (rustc 1.93.0, 2026-01-19)
- No async runtime; all code is synchronous/single-threaded

**Package Manager:**
- Cargo 1.93.0
- Lockfile: present (`Cargo.lock` committed, version 4 format)

## Frameworks

**Core:**
- No web framework - this is a pure numerical/scientific library
- plotters 0.3.7 - SVG chart generation for fit quality visualization

**Testing:**
- Built-in `#[test]` via `cargo test` - integration tests in `tests/` directory
- criterion 0.5.1 (dev-dependency) - Microbenchmarking with HTML reports

**Build/Dev:**
- Cargo (standard Rust build system) - no custom build scripts or proc macros
- No `build.rs` present

## Key Dependencies

**Critical (runtime):**
- plotters 0.3.7 - SVG rendering for market-vs-fit implied volatility plots. Used in `src/fit_common.rs` and multiple binary targets. Only SVG backend is used (`SVGBackend`).

**Critical (dev):**
- criterion 0.5.1 (with `html_reports` feature) - Benchmarking calibration performance. Config in `Cargo.toml` `[[bench]]` section, bench source at `benches/calibration.rs`.

**Zero external math dependencies:**
- No `nalgebra`, `ndarray`, `statrs`, or other numerical crates
- All mathematical primitives are implemented from scratch:
  - `src/math/erf.rs` (251 lines) - Cody's erf/erfc/erfcx with rational Chebyshev approximations
  - `src/math/normal.rs` (130 lines) - Standard normal PDF, CDF, inverse CDF
  - `src/math/normal_hp.rs` (194 lines) - High-precision normal distribution with asymptotic tail expansion
  - `src/math/constants.rs` (74 lines) - Machine-precision thresholds and mathematical constants
  - `src/solver/nelder_mead.rs` (194 lines) - Bounded Nelder-Mead optimizer (derivative-free)
  - `src/solver/brent.rs` (100 lines) - Brent's root-finding method
  - `src/pricing/lets_be_rational.rs` (287 lines) - Peter Jaeckel's "Let's Be Rational" implied vol solver
  - `src/pricing/rational_cubic.rs` (102 lines) - Rational cubic interpolation for IV initial guesses

**Python (scripts only, not packaged):**
- yfinance - Yahoo Finance API client for option chain download
- pandas - DataFrame manipulation for CSV processing

## Configuration

**Environment:**
- No `.env` files present
- No environment variables required for library operation
- Data paths are hardcoded or passed as CLI arguments to binaries

**Build:**
- `Cargo.toml` - Single package manifest, no workspace
- Edition 2024 (latest Rust edition)
- No feature flags defined
- No conditional compilation
- Benchmark harness disabled for criterion (`harness = false`)

**No configuration files detected:**
- No `.rustfmt.toml` (uses default `rustfmt` settings)
- No `clippy.toml` (uses default Clippy settings)
- No `rust-toolchain.toml` (uses system-installed Rust)

## Binaries

The crate produces one library and seven binary targets:

| Binary | Source | Purpose |
|--------|--------|---------|
| `fit_cboe` | `src/bin/fit_cboe.rs` | Fit SSVI surface to CBOE data (implicit theta solver) |
| `fit_crude` | `src/bin/fit_crude.rs` | Fit SSVI surface to CBOE data (4D Nelder-Mead crude solver) |
| `fit_kaggle` | `src/bin/fit_kaggle.rs` | Fit SSVI surface to Kaggle SPY data |
| `fit_real` | `src/bin/fit_real.rs` | Fit SSVI to synthetic market-like data (per-slice) |
| `fit_real_surface` | `src/bin/fit_real_surface.rs` | Fit SSVI surface to synthetic data (with calendar penalty) |
| `plot_kaggle` | `src/bin/plot_kaggle.rs` | Plot raw IV smiles from Kaggle data |
| `report` | `src/bin/report.rs` | Generate SSVI fit quality report across parameter grid |

Usage pattern: `cargo run --bin <name> -- <args>`

## Platform Requirements

**Development:**
- Rust stable toolchain (edition 2024 requires rustc >= 1.85.0)
- Python 3 with `yfinance` and `pandas` (only for data acquisition)
- No OS-specific dependencies; pure Rust with no C FFI or system libraries

**Production:**
- This is a library crate, not a deployed service
- Compiles to native code via `cargo build --release`
- No runtime dependencies beyond the OS (no database, no network)
- Target: any platform supported by Rust stable (Linux, macOS, Windows)

---

*Stack analysis: 2026-03-08*
