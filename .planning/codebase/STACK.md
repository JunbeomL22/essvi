# Technology Stack

**Analysis Date:** 2026-03-05

## Languages

**Primary:**
- Rust (Edition 2024) - All application code

**Secondary:**
- None

## Runtime

**Environment:**
- Rust native binary (compiled)
- rustc 1.92.0 (ded5c06cf 2025-12-08)

**Package Manager:**
- Cargo 1.92.0
- Lockfile: `Cargo.lock` present and committed

## Frameworks

**Core:**
- No web/application framework. This is a pure computational library.

**Testing:**
- Built-in Rust test framework (`#[cfg(test)]`, `#[test]`, `assert_eq!`)
- No external test dependencies

**Build/Dev:**
- Cargo (standard Rust build system)
- No custom build scripts (`build.rs` not present)

## Key Dependencies

**Critical:**
- None. Zero external crate dependencies in `Cargo.toml`.

**Infrastructure:**
- None. The project is entirely self-contained with no third-party crates.

## Domain Context

This is an **SSVI (Surface Stochastic Volatility Inspired) calibration library** for quantitative finance. It implements:
- Bounded Nelder-Mead optimizer (derivative-free nonlinear optimization)
- SSVI implied volatility surface parameterization and calibration
- Brent root-finding for implicit theta computation

The design guideline in `documents/guideline.md` specifies that external dependencies (like `argmin`, `nlopt`) are intentionally avoided in favor of a self-contained ~170-line Nelder-Mead implementation.

## Configuration

**Environment:**
- No environment variables required
- No `.env` files present
- Pure library with no runtime configuration

**Build:**
- `Cargo.toml`: Package manifest (edition 2024, version 0.1.0)
- `Cargo.lock`: Dependency lockfile (trivial, only the package itself)

## Platform Requirements

**Development:**
- Rust toolchain (stable, edition 2024 support requires rustc >= 1.85)
- No platform-specific dependencies

**Production:**
- Compiles to native binary/library on any Rust-supported platform
- No OS-specific code detected

---

*Stack analysis: 2026-03-05*
