# Technology Stack

**Analysis Date:** 2026-03-05

## Languages

**Primary:**
- Rust 1.93.0 - All source code and binaries
  - Edition: 2024

## Runtime

**Environment:**
- Rust toolchain 1.93.0 (rustc + cargo)
- Cargo 1.93.0 - Package manager and build system
- Lockfile: `Cargo.lock` (present)

## Frameworks

**Core:**
- None - Pure Rust library with no web/application frameworks

**Visualization:**
- plotters 0.3.7 - Charts, graphs, and SVG rendering
  - SVG backend for plot output
  - Used in: `src/bin/report.rs` for generating fit quality visualizations

**Testing & Benchmarking:**
- criterion 0.5.1 - Statistical benchmarking framework
  - Features: HTML report generation (`html_reports` feature enabled)
  - Benchmark runner config: `[[bench]]` harness = false
  - Used in: `benches/calibration.rs`

**Build:**
- Cargo - Standard Rust build system

## Key Dependencies

**Critical:**
- plotters 0.3.7 - Generates SVG plots and heatmaps
  - Includes: chrono, font-kit, image, num-traits, lazy_static
  - Used by: `SVGBackend`, `ChartBuilder` in report generation
- criterion 0.5.1 - Performance benchmarking with statistical analysis
  - Includes: rayon (parallel), serde (serialization), plotters (report plots)

**Transitive (plotters ecosystem):**
- image 0.25.6 - Image processing for plot generation
- chrono 0.4.44 - Date/time handling (plot timestamps)
- font-kit 0.14.3 - Font rendering for chart labels
- num-traits - Numerical trait abstractions
- rayon (via criterion) - Data parallelism for benchmarks

**Infrastructure:**
- No external service dependencies (no http, database, async runtime)
- Pure mathematical computation with local file I/O

## Configuration

**Environment:**
- No `.env` files - Zero environment configuration
- Runtime configuration is hardcoded in source code
  - Example: `NelderMeadConfig::default()` in `src/calibration.rs`

**Build:**
- `Cargo.toml` - Manifest with dependencies and bench configuration
  - Package name: essvi
  - Version: 0.1.0
  - Workspace: Single package

**Profiles:**
- Default profiles (debug/release) - No custom optimization settings

## Platform Requirements

**Development:**
- Rust 1.93.0+ (or specified edition 2024 compatible versions)
- Standard C build tools (cc crate for native compilation)
- Platform-specific font libraries (Core Text on macOS, dwrote on Windows, freetype on Linux)

**Production:**
- Standalone binary - No runtime dependencies beyond system libc
- Compiled to native machine code
- Cross-platform: Linux, macOS, Windows (via plotters platform abstraction)

## Compilation

**Binary Targets:**
- Library: `essvi` - Core SSVI calibration algorithms
  - Located: `src/lib.rs`
- Binary: `report` - Report generation tool
  - Located: `src/bin/report.rs`
  - Callable: `cargo run --bin report`

**Benchmark Suite:**
- `calibration` benchmark in `benches/calibration.rs`
- Run: `cargo bench`

## Standard Library Usage

Heavy reliance on:
- `std::fs` - File I/O (create directories, write files)
- `std::io::Write` - Buffered writing for reports
- Built-in numeric operations (f64 math)
- Memory management (no unsafe code detected)

---

*Stack analysis: 2026-03-05*
