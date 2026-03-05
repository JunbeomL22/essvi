# External Integrations

**Analysis Date:** 2026-03-05

## APIs & External Services

**None Detected** - This is a pure computational mathematics library with no external API integrations.

## Data Storage

**Databases:**
- Not used - No database connectivity

**File Storage:**
- Local filesystem only
  - Write operations: `src/bin/report.rs`
    - Creates output directory: `fs::create_dir_all(plot_dir)`
    - Writes markdown report: `fs::File::create(report_path)`
    - Writes report bytes: `file.write_all(md.as_bytes())`
  - No read operations from external sources
  - No cloud storage integration

**Caching:**
- None - No caching layer implemented

## Authentication & Identity

**Auth Provider:**
- Not applicable - Standalone library with no user management

## Monitoring & Observability

**Error Tracking:**
- None - No error tracking service

**Logs:**
- `println!` debugging available but not used in main library
- Benchmark results logged to console via criterion
- No persistent logging infrastructure

**Metrics:**
- criterion framework generates statistical metrics for benchmarks
  - HTML reports saved to filesystem
  - CSV data for manual analysis

## CI/CD & Deployment

**Hosting:**
- Not applicable - This is a library/tool, not a service

**CI Pipeline:**
- Not detected - No GitHub Actions, GitLab CI, or similar

**Build Output:**
- Standalone Rust binary compiled via `cargo build --release`
- Deployable as single executable to any target platform

## Environment Configuration

**Required env vars:**
- None - All configuration is compile-time or runtime hardcoded

**Secrets location:**
- N/A - No secrets used or required

**Example configuration (hardcoded in code):**
```rust
// From src/calibration.rs
let config = NelderMeadConfig::default();
// max_iter: 1000
// tol_f: 1e-12
// tol_x: 1e-12
// alpha: 1.0, gamma: 2.0, rho: 0.5, sigma: 0.5
```

## Webhooks & Callbacks

**Incoming:**
- None - No HTTP server or webhook handlers

**Outgoing:**
- None - No external service callbacks

## Network Configuration

**HTTP/HTTPS:**
- Not used - No network communication

**TCP/UDP Sockets:**
- Not used

## Input/Output Patterns

**Input Sources:**
- Programmatic: Function parameters in library API
- Example from `src/bin/report.rs`:
  - In-memory scenario data structures
  - Market data generated from mathematical functions
  - No external data ingestion

**Output Destinations:**
- SVG files to filesystem (`documents/plots/`)
- Markdown report file (`documents/fit_quality_report.md`)
- Console output for benchmark statistics

## Data Format Conversions

**Serialization:**
- serde + serde_json (transitive via criterion)
  - Only used internally by criterion for benchmark result storage
  - No data exchange format for external systems

**Image Formats:**
- SVG (vector graphics via plotters)
- PNG/GIF support in image crate but not used

## Library/Tool Dependencies Exposure

**Public API (from `src/lib.rs`):**
- `pub mod brent` - Root finding algorithm
- `pub mod calibration` - SSVI parameter calibration
- `pub mod nelder_mead` - Optimization algorithm
- `pub mod ssvi` - Stochastic SVI volatility model

**No external service integrations exposed to library users**

## Edge Cases & Limitations

**File System Assumption:**
- Writable filesystem required at report generation time
- `expect()` panic on write failures in `src/bin/report.rs`
- No retry logic or fallback handling

**Platform Dependencies:**
- plotters requires platform-specific font rendering libraries
  - macOS: Core Foundation, Core Graphics, Core Text
  - Windows: dwrote (DirectWrite)
  - Linux: freetype

---

*Integration audit: 2026-03-05*
