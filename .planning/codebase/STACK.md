# Technology Stack

**Analysis Date:** 2026-03-07

## Languages

**Primary:**
- Rust (edition 2024) - All library code, binaries, tests, and benchmarks

**Secondary:**
- None

## Runtime

**Environment:**
- rustc 1.93.0 (254b59607 2026-01-19)
- No `rust-toolchain.toml` present; depends on system-installed nightly/stable

**Package Manager:**
- Cargo 1.93.0 (083ac5135 2025-12-15)
- Lockfile: `Cargo.lock` present and committed

## Frameworks

**Core:**
- No web/application framework. This is a pure numerical/scientific Rust library.

**Testing:**
- Built-in `#[cfg(test)]` / `#[test]` - Unit tests co-located in source modules
- Integration tests in `tests/` directory
- criterion 0.5 (with `html_reports` feature) - Benchmarking framework

**Build/Dev:**
- Cargo (standard Rust build system) - Build, test, bench, run binaries

## Key Dependencies

**Critical:**
- plotters 0.3 - SVG chart generation for fit quality reports and visualization. Used in all three binaries (`src/bin/report.rs`, `src/bin/fit_real.rs`, `src/bin/fit_real_surface.rs`) to produce implied volatility fit plots.

**Dev/Bench:**
- criterion 0.5 - Performance benchmarking for calibration routines. Config: `benches/calibration.rs`, harness disabled via `[[bench]]` in `Cargo.toml`.

**Zero external numerical dependencies:**
- All numerical algorithms (Nelder-Mead optimizer, Brent root finding, Newton solver, SSVI model) are implemented from scratch in the library. No dependency on nalgebra, ndarray, or similar.

## Configuration

**Environment:**
- No `.env` files. No runtime configuration files.
- No environment variables consumed at runtime.
- All parameters are hardcoded in source or passed as function arguments.

**Build:**
- `Cargo.toml` - Package manifest and dependency specification
- No `.rustfmt.toml`, `clippy.toml`, or `rust-toolchain.toml` present
- `.gitignore` contains only `/target`

## Binaries

Three standalone binaries defined in `src/bin/`:

| Binary | Source | Purpose |
|--------|--------|---------|
| `report` | `src/bin/report.rs` | Parameter grid sweep: T x skew slope fit quality report |
| `fit_real` | `src/bin/fit_real.rs` | Per-slice SSVI fit to synthetic market data (12 expiries) |
| `fit_real_surface` | `src/bin/fit_real_surface.rs` | Surface fit with calendar arbitrage penalty |

Run with:
```bash
cargo run --bin report
cargo run --bin fit_real
cargo run --bin fit_real_surface
```

All binaries write output to `documents/plots/` (SVG files) and `documents/` (Markdown reports).

## Platform Requirements

**Development:**
- Rust 1.93.0+ (edition 2024 features required)
- No OS-specific dependencies; pure Rust with no C/FFI bindings
- No GPU or SIMD requirements

**Production:**
- Library crate (`essvi`) - embeddable in any Rust project
- No deployment target; this is a library + CLI toolset, not a service

---

*Stack analysis: 2026-03-07*
