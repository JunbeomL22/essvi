# External Integrations

**Analysis Date:** 2026-03-07

## APIs & External Services

**None.** This is a self-contained numerical library with no network calls, no API clients, and no external service dependencies.

## Data Storage

**Databases:**
- None. No database of any kind.

**File Storage:**
- Local filesystem only
  - Binaries write SVG plots to `documents/plots/`
  - Binaries write Markdown reports to `documents/`
  - Uses `std::fs` for all file I/O

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- Not applicable. No authentication of any kind.

## Monitoring & Observability

**Error Tracking:**
- None. Errors are handled via `Option<T>` return types and `eprintln!` for binary error output.

**Logs:**
- `println!` / `eprintln!` in binaries only. No logging framework.
- Library code uses no logging whatsoever; it returns `Option<CalibrationResult>`.

## CI/CD & Deployment

**Hosting:**
- Not applicable. Library crate, not a deployed service.

**CI Pipeline:**
- None detected. No `.github/workflows/`, `.gitlab-ci.yml`, or similar CI configuration.

## Environment Configuration

**Required env vars:**
- None. The library and binaries require no environment variables.

**Secrets location:**
- Not applicable. No secrets, API keys, or credentials of any kind.

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Output Artifacts

The only "integration" is filesystem output from the binaries:

| Binary | Output Files | Format |
|--------|-------------|--------|
| `report` | `documents/fit_quality_report.md`, `documents/plots/fit_*.svg` | Markdown + SVG |
| `fit_real` | `documents/real-world-fit.md`, `documents/plots/fit_real_T*.svg` | Markdown + SVG |
| `fit_real_surface` | `documents/real-world-surface-fit.md`, `documents/plots/fit_surface_T*.svg` | Markdown + SVG |

## Third-Party Library Integration

**plotters 0.3:**
- Used exclusively in `src/bin/report.rs`, `src/bin/fit_real.rs`, `src/bin/fit_real_surface.rs`
- SVGBackend for chart rendering (no bitmap/PNG output)
- Not used in the library core (`src/lib.rs`, `src/ssvi.rs`, `src/calibration.rs`, etc.)
- Patterns used: `ChartBuilder`, `LineSeries`, `Circle`, `SVGBackend`

**criterion 0.5:**
- Dev-dependency only, used in `benches/calibration.rs`
- Benchmarks: `calibrate_20pt_slice`, `solve_theta`, `total_variance_20pt`, `surface_12_slices`
- HTML reports enabled via feature flag

## Data Sources

All market data used by the binaries is **synthetic** (generated in-code):
- `src/bin/fit_real.rs` - `build_market_slices()` generates 12 synthetic equity-index-like slices
- `src/bin/fit_real_surface.rs` - Same synthetic data, different calibration approach
- `src/bin/report.rs` - `make_market_data()` generates parametric smile data
- No CSV/JSON/external data file parsing

Reference materials exist in `documents/` (PDFs, PNGs) but are not read by any code.

---

*Integration audit: 2026-03-07*
