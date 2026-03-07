# Stack Research: Option Data Storage and Format

**Domain:** European-style index option data collection and storage for SVI calibration
**Researched:** 2026-03-07
**Confidence:** HIGH (core format conventions) / MEDIUM (free data source availability)

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Plain CSV | N/A | Option chain data storage | Universal interchange format; every exchange, vendor, and academic tool exports CSV. No binary format lock-in. The project's pure-Rust constraint rules out databases. Human-readable, git-diffable, trivially parseable. |
| UTF-8 encoding | N/A | File encoding | Standard, no BOM issues, compatible with all Rust CSV parsers. |
| ISO 8601 dates | N/A | Date format (`YYYY-MM-DD`) | Sortable, unambiguous, international standard. Used by CBOE, OptionMetrics, and optiondata.org. |

### Future Parser Dependencies (v1.3+, NOT this milestone)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `csv` crate | 1.4 | CSV reading/writing | BurntSushi's csv crate is the de-facto Rust CSV parser. Supports serde-based typed deserialization, streaming reads, handles quoting/escaping edge cases. Battle-tested, zero-unsafe. |
| `serde` + `serde_derive` | 1.x | Typed deserialization | Rust ecosystem standard. Enables `#[derive(Deserialize)]` on option record structs, mapping CSV headers to struct fields automatically. |
| `chrono` | 0.4 | Date parsing/formatting | Standard Rust date library. Needed for parsing expiration dates, computing days-to-expiry (DTE), and time-to-maturity (tau). `NaiveDate` type matches option data conventions (dates without timezones). |
| Existing `lets_be_rational` | In-tree | IV from option prices | Already implemented. Machine-precision implied volatility recovery. No external dependency. |
| Existing `black76` | In-tree | Forward inference | Already implemented. Can compute forward from put-call parity on matched call/put pairs. |

### Infrastructure (Unchanged)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust | Edition 2024 | Language | Already the project language. Pure Rust constraint. |
| Cargo | Standard | Build system | Already in use. |
| Git | Standard | Version control | Data files tracked in git (small, curated datasets). |

### Data Collection Tools (External, One-Time Use)

| Tool | Purpose | Notes |
|------|---------|-------|
| Python + yfinance | One-time data download | Use to pull option chains from Yahoo Finance. Not a runtime dependency -- used once to create CSV files checked into `data/`. |
| Manual browser download | Fallback data collection | CBOE marking prices and Eurex settlement data are available as direct CSV downloads from exchange websites. |

## CSV Column Conventions

### Recommended Canonical Format (Long Format)

Use **long format** (one row per option contract) rather than wide format. This is the dominant convention across all major data sources (CBOE DataShop, OptionMetrics IvyDB, Yahoo Finance, optiondata.org). Long format is simpler to parse, filter, and extend.

```csv
date,underlying,expiry,strike,option_type,bid,ask,mid,volume,open_interest,underlying_price
2026-03-06,SX5E,2026-03-20,4700.0,C,185.20,187.40,186.30,1234,5678,4850.5
2026-03-06,SX5E,2026-03-20,4700.0,P,35.80,37.60,36.70,892,3456,4850.5
2026-03-06,SX5E,2026-03-20,4750.0,C,152.30,154.10,153.20,567,2345,4850.5
```

### Column Definitions

| Column | Type | Format | Description | Required |
|--------|------|--------|-------------|----------|
| `date` | string | `YYYY-MM-DD` | Quote/observation date | YES |
| `underlying` | string | ticker symbol | Underlying index (e.g., `SX5E`, `SPX`, `NKY`) | YES |
| `expiry` | string | `YYYY-MM-DD` | Option expiration date | YES |
| `strike` | f64 | decimal | Strike price in index points | YES |
| `option_type` | string | `C` or `P` | Call or Put | YES |
| `bid` | f64 | decimal | Best bid price (in index points) | YES |
| `ask` | f64 | decimal | Best ask price (in index points) | YES |
| `mid` | f64 | decimal | Mid price = (bid + ask) / 2 | Derived (include for convenience) |
| `volume` | u64 | integer | Contracts traded on this date | Recommended |
| `open_interest` | u64 | integer | Outstanding contracts at close | Recommended |
| `underlying_price` | f64 | decimal | Underlying index level at snapshot time | YES (needed for moneyness) |

### Optional Extended Columns

| Column | Type | Description | When to Include |
|--------|------|-------------|-----------------|
| `implied_volatility` | f64 | Exchange-computed IV (decimal, not %) | If available from source (useful for cross-validation against our solver) |
| `settlement_price` | f64 | Official settlement/closing price | If available instead of bid/ask |
| `risk_free_rate` | f64 | Annualized risk-free rate (decimal) | If available from source |
| `dividend_yield` | f64 | Annualized dividend yield (decimal) | If available from source |

### Why These Column Names

Column names follow the intersection of conventions across major data providers:

- **CBOE DataShop** uses: `quote_date`, `expiration`, `strike`, `option_type`, `last_bid_price`, `last_ask_price`, `underlying_close`, `open_interest`, `total_volume`
- **OptionMetrics IvyDB** (academic gold standard, 300+ institutions) uses: `date`, `exdate`, `strike_price`, `cp_flag`, `best_bid`, `best_offer`, `volume`, `open_interest`, `impl_volatility`
- **Yahoo Finance / yfinance** uses: `strike`, `bid`, `ask`, `volume`, `openInterest`, `impliedVolatility`
- **optiondata.org** uses: `quote_date`, `expiration`, `strike`, `type`, `bid`, `ask`, `volume`, `open_interest`, `implied_volatility`

The recommended names are a simplified, lowercase-with-underscores normalization that maps cleanly to all sources. They match what any quant would expect.

### Rust Struct Mapping (for future parser)

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OptionRecord {
    pub date: String,           // "YYYY-MM-DD"
    pub underlying: String,     // "SX5E", "SPX", "NKY"
    pub expiry: String,         // "YYYY-MM-DD"
    pub strike: f64,
    pub option_type: String,    // "C" or "P"
    pub bid: f64,
    pub ask: f64,
    pub mid: f64,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub volume: Option<u64>,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub open_interest: Option<u64>,
    pub underlying_price: f64,
}
```

## File Organization

### Recommended: One File Per Observation Date

```
data/
  options/
    SX5E/
      2026-03-06.csv      # All strikes and expiries for this date
      2026-03-07.csv
    SPX/
      2026-03-06.csv
  metadata/
    SX5E.json             # Index-level metadata
    SPX.json
  README.md               # Documents data sources, collection dates, known issues
```

**Why one file per date (not per expiry):**

1. **Matches data acquisition**: You download/scrape a full option chain for a given day. One download = one file.
2. **Calibration workflow**: SVI calibration fits a surface to a single day's data across multiple expiries. One file = one calibration input.
3. **Simpler to manage**: A trading day has 1 file instead of N files (one per expiry). With 10 expiries per day, per-expiry organization creates 10x file count.
4. **Industry standard**: CBOE DataShop, OptionMetrics, and most vendors organize by observation date. Academic datasets (IvyDB) are keyed by date.
5. **Git-friendly**: One file per day means adding data for a new date is one commit adding one file.

### Metadata File

A lightweight JSON sidecar describing the index and data provenance:

```json
{
    "underlying": "SPX",
    "name": "S&P 500 Index",
    "exchange": "CBOE",
    "exercise_style": "European",
    "settlement": "cash",
    "currency": "USD",
    "source": "Yahoo Finance (^SPX)",
    "collection_method": "yfinance Python script",
    "collection_date": "2026-03-07",
    "notes": "AM-settled, European-style exercise"
}
```

## Data Sources (Ranked by Practicality for v1.2)

### 1. Yahoo Finance via yfinance -- RECOMMENDED

**What:** Option chains for SPX (^SPX) or Euro Stoxx 50 ETF proxy (FEZ)
**Fields provided:** contractSymbol, strike, bid, ask, lastPrice, volume, openInterest, impliedVolatility, inTheMoney
**Access:** Free, no API key needed, Python one-liner
**Confidence:** HIGH -- verified from multiple sources and the yfinance documentation

```python
import yfinance as yf
import pandas as pd

ticker = yf.Ticker("^SPX")  # European-style, cash-settled
for exp in ticker.options:
    chain = ticker.option_chain(exp)
    calls = chain.calls.assign(option_type='C')
    puts = chain.puts.assign(option_type='P')
    # Normalize column names, combine, save as CSV
```

**Limitations:** Data is delayed 15-20 minutes (fine for EOD snapshots). Euro Stoxx 50 direct index options not available -- only ETF proxies (FEZ). SPX is the better target.

### 2. CBOE Marking Prices (Free, SPX only)

**What:** Daily 3:00 PM CT indicative marking prices for SPX/SPXW options
**Fields provided:** Marking price, BBO (bid/ask) at snapshot time
**Access:** Free CSV download from cboe.com, no account needed
**Confidence:** MEDIUM -- page confirmed, CSV files exist, but exact column format not independently verified

### 3. Eurex Settlement Prices (Euro Stoxx 50)

**What:** Official daily settlement prices for OESX options
**Access:** Available on eurex.com, may require registration
**Limitations:** Settlement prices only (no bid/ask spread). Manual download.
**Confidence:** MEDIUM -- settlement prices confirmed available, exact CSV format not verified

### 4. JPX Settlement Prices (Nikkei 225)

**What:** Daily settlement prices for Nikkei 225 options on Osaka Exchange
**Access:** CSV download on jpx.co.jp
**Limitations:** Japanese language documentation. Settlement prices only.
**Confidence:** LOW -- data existence confirmed but format details not verified

### Why SPX Over Euro Stoxx 50 for v1.2

1. **SPX options are European-style and cash-settled** -- exactly what the project needs for Black-76 pricing
2. yfinance provides bid, ask, volume, OI, and exchange-computed IV all in one pull
3. SPX has the deepest liquidity across many strikes and expiries, ideal for calibration testing
4. Euro Stoxx 50 via Yahoo Finance only provides ETF proxies (FEZ), not direct index options
5. Direct Eurex data requires registration or commercial subscriptions

Euro Stoxx 50 can be added later from Eurex settlement data if needed.

## Downstream Integration: CSV to Calibration Pipeline

The existing calibration pipeline (`CalibrationInput`) expects:
- `k_slice: &[f64]` -- log-moneyness values
- `w_market: &[f64]` -- total implied variance (sigma^2 * T)
- `theta_star: f64` -- ATM total variance
- `k_star: f64` -- ATM log-moneyness reference

Converting from CSV option records to calibration input requires:

1. **Group by expiry** -- each expiry becomes one SSVI slice
2. **Compute forward price** -- F = S * exp((r - q) * T), or approximate from put-call parity on matched call/put pairs at the same strike
3. **Compute log-moneyness** -- k = ln(K / F) for each strike K
4. **Compute implied volatility** -- use the project's Let's Be Rational solver on mid prices, OR use exchange-provided IV if available
5. **Compute total variance** -- w = sigma^2 * T

This conversion logic belongs in a future milestone. The CSV format defined here provides all fields needed for this conversion.

## Installation

### No New Dependencies for v1.2

The v1.2 milestone is data collection only. No code changes, no new crate dependencies. The `data/` directory is a pure filesystem addition.

### Future Dependencies (v1.3+ parser)

```toml
# Cargo.toml additions for CSV parsing (future milestone)
[dependencies]
csv = "1.4"
serde = { version = "1", features = ["derive"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
```

Note: `csv` brings in `serde` as a dependency. `chrono` with `default-features = false` avoids the timezone database, keeping it lean.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| CSV (plain text) | Parquet / Arrow | Large datasets (millions of rows), columnar queries. Overkill for a few days of option chains (~500-2000 rows per day). |
| CSV (plain text) | SQLite | Need ad-hoc queries, multi-table joins. Adds binary dependency, not human-readable, not git-diffable. |
| CSV (plain text) | JSON | Nested data structures. Option chains are flat/tabular -- CSV is more natural and compact. |
| One file per date | One file per expiry | Multi-year historical analysis. Not relevant for v1.2 scope (a few days of data). |
| Long format (one row per contract) | Wide format (C_BID/C_ASK/P_BID/P_ASK columns) | Visual comparison of calls vs puts at same strike. Harder to parse, wastes space when only calls or only puts exist at a strike. |
| yfinance (Yahoo Finance) | Bloomberg / Refinitiv | Institutional-grade data with dividend/rate curves. Requires expensive subscriptions. |
| yfinance (Yahoo Finance) | CBOE DataShop (paid) | Full historical EOD with Greeks. Costs money. |
| `csv` crate | Manual `str::split` | Never. Error-prone, no quoting support, no serde integration. |
| `csv` crate | `polars` | Massive dataframe dependency for reading a few thousand rows. |
| `chrono` | `time` crate | `chrono` has wider ecosystem adoption and more parsing flexibility for financial date formats. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Binary/proprietary formats (HDF5, Feather) | Adds dependencies, not human-readable, not git-diffable | Plain CSV |
| Database (SQLite, Postgres) | Overkill for ~5 CSV files; violates pure-Rust spirit; adds operational complexity | CSV files in `data/` directory |
| Real-time API feeds | v1.2 needs static snapshots, not streaming data. Adds async/networking complexity. | One-time download scripts |
| Wide format CSV | Harder to parse, requires complex deserialization, wastes space | Long format (one row per contract) |
| Custom delimiters (TSV, pipe-delimited) | CSV is universal, no reason to deviate | Standard comma-separated CSV |
| BOM-marked UTF-8 | Causes parsing issues with many tools | Plain UTF-8 without BOM |

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| csv 1.4 | serde 1.x | Built-in serde support via `csv::Reader::deserialize()` |
| csv 1.4 | Rust edition 2024 | No known compatibility issues |
| chrono 0.4 | serde 1.x | Enable `features = ["serde"]` for date deserialization from CSV |
| serde 1.x | csv 1.4 + chrono 0.4 | Central to the typed deserialization pipeline |
| plotters 0.3 | N/A | Already in project, unaffected by data format choices |

## Sources

- [CBOE DataShop -- Option EOD Summary](https://datashop.cboe.com/option-eod-summary) -- verified column names: quote_date, underlying_symbol, expiration, strike, option_type, bid, ask, volume, open_interest (HIGH confidence)
- [CBOE DataShop -- End of Day Options Summary (Legacy)](https://datashop.cboe.com/end-of-day-options-summary) -- verified 17-column format: quote_date, underlying_symbol, root, expiry, strike, type, open_interest, total_volume, high, low, open, last, last_bid_price, last_ask_price, underlying_close, series_type, product_type (HIGH confidence)
- [CBOE Marking Prices](https://www.cboe.com/us/options/market_statistics/proprietary_index_marking_prices/) -- free CSV with marking price and BBO, updated daily (MEDIUM confidence on exact column headers)
- [OptionMetrics IvyDB](https://optionmetrics.com/united-states/) -- academic gold standard; secid, date, exdate, cp_flag, strike_price, best_bid, best_offer, volume, open_interest, impl_volatility (HIGH confidence)
- [Yahoo Finance / yfinance](https://www.macroption.com/yahoo-finance-options-python/) -- columns: contractSymbol, lastTradeDate, strike, lastPrice, bid, ask, change, percentChange, volume, openInterest, impliedVolatility, inTheMoney, contractSize, currency (HIGH confidence)
- [optiondata.org](https://optiondata.org/) -- verified columns: contract, underlying, expiration, type, strike, style, bid, bid_size, ask, ask_size, volume, open_interest, quote_date, delta, gamma, theta, vega, implied_volatility (HIGH confidence)
- [rfbressan/py_ssvi](https://github.com/rfbressan/py_ssvi) -- SVI calibration uses: period, moneyness/k, iv, tau, w columns (HIGH confidence)
- [BurntSushi/rust-csv](https://github.com/BurntSushi/rust-csv) -- csv crate v1.4, serde support, `csv::invalid_option` for optional fields (HIGH confidence)
- [Eurex Euro Stoxx 50 Options](https://www.eurex.com/ex-en/markets/idx/stx/euro-stoxx-50-derivatives/products/EURO-STOXX-50-Index-Options-46548) -- European-style, cash-settled, settlement via Black-76 (HIGH confidence)
- [JPX Nikkei 225 Options](https://www.jpx.co.jp/english/derivatives/products/domestic/225options/01.html) -- settlement prices available as CSV download (LOW confidence on format details)

---
*Stack research for: option data collection and storage for SVI calibration*
*Researched: 2026-03-07*
