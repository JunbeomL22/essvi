# Codebase Structure

**Analysis Date:** 2026-03-08

## Directory Layout

```
essvi/
├── benches/                # Criterion benchmarks
│   └── calibration.rs      # Calibration + surface benchmarks
├── data/                   # Market data (CSV files)
│   ├── cboe/               # CBOE index options
│   │   ├── ndx/            # Nasdaq 100 (date.csv)
│   │   └── spx/            # S&P 500 (date.csv)
│   ├── eurex/              # Eurex options (placeholder)
│   │   └── sx5e/           # Euro Stoxx 50 (.gitkeep only)
│   ├── kaggle/             # Historical Kaggle datasets
│   │   └── spy/            # SPY options (date.csv)
│   └── sample/             # Sample data (.gitkeep only)
├── documents/              # Generated reports and plots
│   ├── data-plots/         # Per-dataset fit plots (SVG)
│   └── mock-results/       # Reference/mock results
├── scripts/                # Data acquisition utilities
│   └── fetch_options.py    # Yahoo Finance option chain fetcher
├── src/                    # Rust source code
│   ├── bin/                # Binary entry points
│   │   ├── fit_cboe.rs     # Fit SSVI to CBOE data (implicit theta)
│   │   ├── fit_crude.rs    # Fit SSVI to CBOE data (crude 4D solver)
│   │   ├── fit_kaggle.rs   # Fit SSVI to Kaggle SPY data
│   │   ├── fit_real.rs     # Fit SSVI to synthetic data (per-slice)
│   │   ├── fit_real_surface.rs  # Fit SSVI surface to synthetic data
│   │   ├── plot_kaggle.rs  # Plot raw Kaggle IV smiles (no fitting)
│   │   └── report.rs       # Parameter grid quality report
│   ├── math/               # Mathematical primitives
│   │   ├── mod.rs          # Module declarations
│   │   ├── constants.rs    # Machine-precision & math constants
│   │   ├── erf.rs          # Error function (erf, erfc, erfcx)
│   │   ├── normal.rs       # Normal PDF, CDF, inverse CDF
│   │   └── normal_hp.rs    # High-precision normal CDF for tails
│   ├── model/              # Volatility model definitions
│   │   ├── mod.rs          # Module declarations
│   │   └── ssvi.rs         # SSVI phi, total_variance, no-arb check
│   ├── pricing/            # Option pricing
│   │   ├── mod.rs          # Module declarations
│   │   ├── error.rs        # PricingError enum
│   │   ├── black76.rs      # Black-76 price, greeks, discounted price
│   │   ├── lets_be_rational.rs  # Implied vol solver (LBR algorithm)
│   │   └── rational_cubic.rs    # Rational cubic interpolation
│   ├── solver/             # Numerical optimization
│   │   ├── mod.rs          # Module declarations
│   │   ├── brent.rs        # Brent's root-finding method
│   │   └── nelder_mead.rs  # Bounded Nelder-Mead optimizer
│   ├── calibration.rs      # Implicit-theta SSVI calibration
│   ├── crude_solver.rs     # Direct 4D SSVI calibration
│   ├── fit_common.rs       # Shared types/helpers for fit binaries
│   └── lib.rs              # Library root (module declarations + re-exports)
├── tests/                  # Integration tests
│   ├── brent.rs            # Brent solver tests
│   ├── calibration.rs      # Calibration pipeline tests
│   ├── crude_solver.rs     # Crude solver tests
│   ├── implied_vol.rs      # Implied vol round-trip tests
│   ├── math.rs             # Math function accuracy tests
│   ├── nelder_mead.rs      # Nelder-Mead optimizer tests
│   ├── pricing.rs          # Black-76 pricing tests
│   ├── ssvi.rs             # SSVI model tests
│   └── steep_skew.rs       # Edge case: steep skew calibration
├── .planning/              # GSD planning documents
├── Cargo.toml              # Package manifest
├── Cargo.lock              # Dependency lockfile
├── .gitignore              # Only ignores /target
└── README.md               # Minimal placeholder
```

## Directory Purposes

**`src/`:**
- Purpose: All Rust source code for the library and binaries
- Contains: Library modules (organized by domain) and binary entry points
- Key files: `lib.rs` (library root), `calibration.rs` and `crude_solver.rs` (core calibration logic)

**`src/bin/`:**
- Purpose: CLI binary entry points that consume the library
- Contains: Seven standalone binaries, each with `fn main()`
- Pattern: Each binary follows parse -> fit -> report structure
- Key files: `fit_cboe.rs` (primary real-data fitting tool), `fit_crude.rs` (alternative solver)

**`src/math/`:**
- Purpose: Machine-precision mathematical building blocks
- Contains: Error functions, normal distribution, constants
- Key files: `erf.rs` (Cody's rational Chebyshev), `normal_hp.rs` (tail-safe CDF)

**`src/model/`:**
- Purpose: Parametric volatility model definitions
- Contains: SSVI model only (currently)
- Key files: `ssvi.rs` (phi, total_variance, no_arbitrage_satisfied)

**`src/pricing/`:**
- Purpose: Option pricing and implied volatility
- Contains: Black-76 model, Let's Be Rational IV solver, error types
- Key files: `black76.rs` (price + all greeks), `lets_be_rational.rs` (IV solver)

**`src/solver/`:**
- Purpose: Generic numerical solvers (model-agnostic)
- Contains: Nelder-Mead optimizer, Brent root finder
- Key files: `nelder_mead.rs` (used by both calibration approaches)

**`data/`:**
- Purpose: Market data input files
- Contains: CSV files organized by source/ticker/date
- Key structure: `data/<source>/<ticker>/<date>.csv`
- Not generated; committed to repo (some files tracked, some untracked)

**`documents/`:**
- Purpose: Generated reports, plots, and reference materials
- Contains: SVG plots, markdown reports, reference images
- Generated by fit binaries; partially committed

**`tests/`:**
- Purpose: Integration tests (one file per module)
- Contains: 9 test files covering all library modules
- Pattern: Named to match the module being tested

**`benches/`:**
- Purpose: Performance benchmarks using Criterion
- Contains: Single file with 4 benchmark functions
- Key file: `calibration.rs` (benchmarks calibrate, solve_theta, total_variance, surface)

**`scripts/`:**
- Purpose: Data acquisition helper scripts (Python)
- Contains: `fetch_options.py` for downloading option chains from Yahoo Finance

## Key File Locations

**Entry Points:**
- `src/lib.rs`: Library root -- declares all public modules and backward-compatible re-exports
- `src/bin/fit_cboe.rs`: Primary real-data calibration binary (implicit theta solver)
- `src/bin/fit_crude.rs`: Alternative calibration binary (crude 4D solver)

**Configuration:**
- `Cargo.toml`: Package manifest (edition 2024, only dep: plotters 0.3, dev-dep: criterion 0.5)
- `.gitignore`: Only ignores `/target`

**Core Logic:**
- `src/calibration.rs`: Primary calibration pipeline -- `CalibrationConfig`, `solve_theta`, `calibrate`, `calibrate_with_calendar_penalty`
- `src/crude_solver.rs`: Alternative calibration -- `CrudeCalibConfig`, `calibrate_slice`, `calibrate_surface`, butterfly penalty
- `src/model/ssvi.rs`: SSVI model formulas -- `phi`, `total_variance`, `total_variance_slice`, `no_arbitrage_satisfied`
- `src/solver/nelder_mead.rs`: Bounded Nelder-Mead optimizer -- `nelder_mead_bounded`
- `src/fit_common.rs`: Shared binary utilities -- `FitResult`, `SliceData`, `plot_fit`, `build_market_slices`

**Testing:**
- `tests/calibration.rs`: Calibration pipeline integration tests
- `tests/crude_solver.rs`: Crude solver integration tests
- `tests/pricing.rs`: Black-76 pricing accuracy tests
- `tests/implied_vol.rs`: IV round-trip tests
- `benches/calibration.rs`: Performance benchmarks

**Data:**
- `data/cboe/spx/*.csv`: CBOE SPX option chain CSVs (columns: quote_date, expiry, strike, underlying_price, c_bid, c_ask, c_iv, p_bid, p_ask, p_iv, combined_futures, log_moneyness, dte, total_variance)
- `data/kaggle/spy/*.csv`: Kaggle SPY option CSVs (columns: QUOTE_UNIXTIME, ..., C_IV, P_IV, STRIKE, LOG_MONEYNESS, TOTAL_VARIANCE, DTE, etc.)

## Naming Conventions

**Files:**
- Library modules: `snake_case.rs` (e.g., `nelder_mead.rs`, `crude_solver.rs`, `lets_be_rational.rs`)
- Binaries: `snake_case.rs` with verb-noun pattern (e.g., `fit_cboe.rs`, `plot_kaggle.rs`)
- Tests: `snake_case.rs` matching the module name (e.g., `tests/calibration.rs` tests `src/calibration.rs`)
- Data files: `<date>.csv` within source/ticker directories

**Directories:**
- Source modules: `snake_case` matching Rust module names (e.g., `math/`, `model/`, `pricing/`, `solver/`)
- Data directories: `<source>/<ticker>/` (e.g., `cboe/spx/`, `kaggle/spy/`)

**Modules:**
- `mod.rs` files contain only submodule declarations (e.g., `pub mod ssvi;`)
- Module re-exports in `lib.rs` for backward compatibility: `pub use model::ssvi;`

## Where to Add New Code

**New Volatility Model (e.g., SVI, SABR):**
- Create: `src/model/<name>.rs` (model formulas)
- Register: Add `pub mod <name>;` in `src/model/mod.rs`
- Optionally re-export: Add `pub use model::<name>;` in `src/lib.rs`
- Create calibration: Add calibration functions in a new `src/<name>_calibration.rs` or extend `src/calibration.rs`
- Tests: Add `tests/<name>.rs`

**New Numerical Solver:**
- Create: `src/solver/<name>.rs`
- Register: Add `pub mod <name>;` in `src/solver/mod.rs`
- Tests: Add `tests/<name>.rs`

**New Data Source Binary:**
- Create: `src/bin/fit_<source>.rs` following the parse -> fit -> report pattern from `fit_cboe.rs`
- Reuse: `src/fit_common.rs` for `FitResult`, `plot_fit`
- Data: Add CSV files to `data/<source>/<ticker>/`
- No Cargo.toml changes needed -- Rust auto-discovers binaries in `src/bin/`

**New Math Primitive:**
- Create: `src/math/<name>.rs`
- Register: Add `pub mod <name>;` in `src/math/mod.rs`
- Tests: Extend `tests/math.rs` or create `tests/<name>.rs`

**New Pricing Model:**
- Create: `src/pricing/<name>.rs`
- Register: Add `pub mod <name>;` in `src/pricing/mod.rs`
- Tests: Extend `tests/pricing.rs` or create dedicated test file

**New Benchmark:**
- Add benchmark function in `benches/calibration.rs`
- Register in `criterion_group!` macro at bottom of file

## Special Directories

**`target/`:**
- Purpose: Rust build artifacts (debug/release binaries, dependencies)
- Generated: Yes (by `cargo build`)
- Committed: No (in `.gitignore`)

**`.planning/`:**
- Purpose: GSD planning and codebase analysis documents
- Generated: Partially (by GSD commands)
- Committed: Yes

**`documents/data-plots/`:**
- Purpose: SVG plots generated by fit binaries, organized by dataset
- Generated: Yes (by `cargo run --bin fit_*`)
- Committed: Yes (some subdirectories tracked)

**`documents/mock-results/`:**
- Purpose: Reference results and plots for validation
- Generated: No (manually curated)
- Committed: Yes

---

*Structure analysis: 2026-03-08*
