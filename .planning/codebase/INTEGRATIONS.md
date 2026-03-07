# External Integrations

**Analysis Date:** 2026-03-08

## APIs & External Services

**Yahoo Finance (data acquisition only):**
- Used for downloading European-style index option chains (SPX, NDX)
- SDK/Client: `yfinance` Python package (called from `scripts/fetch_options.py`)
- Auth: None required (public API, no API key)
- Rate limiting: 0.1s between expiry fetches, 2s between tickers (implemented in script)
- Method: `yf.Ticker(symbol).option_chain(expiry)` for each available expiration date
- Tickers configured: `^SPX` (S&P 500), `^NDX` (Nasdaq 100)
- Not used at runtime by the Rust library; data is pre-fetched to CSV files

**No other external APIs:**
- The Rust library (`essvi`) makes zero network calls
- All computation is purely local, operating on CSV files in `data/`

## Data Storage

**Databases:**
- None. No database of any kind (SQL, NoSQL, embedded).

**File Storage:**
- Local filesystem only
- Input data: CSV files in `data/{source}/{underlying}/{YYYY-MM-DD}.csv`
- Output: SVG plots and Markdown reports written to `documents/data-plots/`

**Data directory structure:**
```
data/
  cboe/
    spx/          # S&P 500 option chains
    ndx/          # Nasdaq 100 option chains
  eurex/
    sx5e/         # Euro Stoxx 50 (placeholder, no data yet)
  kaggle/
    spy/          # Kaggle SPY historical data
  sample/         # Synthetic test data (placeholder)
```

**CSV schema (canonical):**
- Required columns: `quote_date`, `expiry`, `strike`, `option_type`, `bid`, `ask`, `underlying_price`
- Optional columns: `volume`, `open_interest`, `implied_vol`
- Schema documented in `data/README.md`

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- Not applicable. No authentication system.
- This is a numerical library, not a web application.

## Monitoring & Observability

**Error Tracking:**
- None. No Sentry, Datadog, or similar.

**Logs:**
- `println!` / `eprintln!` for binary output (progress, diagnostics, results)
- No structured logging framework (no `tracing`, `log`, `env_logger`)
- Library code (`src/lib.rs` and modules) does not emit any log output; only binaries in `src/bin/` print to stdout/stderr

## CI/CD & Deployment

**Hosting:**
- Not deployed. Library crate intended for local use and potential crates.io publication.

**CI Pipeline:**
- None detected. No `.github/workflows/`, no `.gitlab-ci.yml`, no `Makefile`, no CI configuration files.

## Environment Configuration

**Required env vars:**
- None. The library and binaries require no environment variables.

**Secrets location:**
- No secrets needed. Yahoo Finance API is public/unauthenticated.

**CLI arguments for binaries:**
- `fit_cboe`: `cargo run --bin fit_cboe -- <ticker> <date>` (e.g., `spx 2026-03-07`)
- `fit_crude`: `cargo run --bin fit_crude -- <ticker> <date>`
- `fit_kaggle`: `cargo run --bin fit_kaggle -- <date>`
- `plot_kaggle`: No args (processes all files in `data/kaggle/spy/`)
- `fit_real`, `fit_real_surface`, `report`: No args (use built-in synthetic data)

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Data Flow Summary

The only external integration is the data acquisition pipeline:

1. `scripts/fetch_options.py` calls Yahoo Finance API via `yfinance`
2. Option chain data is written to `data/cboe/{ticker}/{date}.csv`
3. Rust binaries read CSV files from disk, perform SSVI calibration
4. Results are written as SVG plots and Markdown reports to `documents/`

All computation is offline. The Rust library has no network dependencies.

---

*Integration audit: 2026-03-08*
