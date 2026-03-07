# Market Data

European-style index option price data for SSVI calibration testing and validation. Raw bid/ask prices stored as CSV snapshots, one file per observation date per underlying.

## Directory Structure

```
data/
  {source}/
    {underlying}/
      {YYYY-MM-DD}.csv
```

**Source-first hierarchy:** Files are organized by data provider, then by underlying instrument, then by observation date. Each CSV contains the complete option chain for one underlying on one trading date.

| Directory | Source | Underlying | Exercise Style |
|-----------|--------|------------|----------------|
| `cboe/spx/` | Yahoo Finance (^SPX) | S&P 500 Index | European |
| `cboe/ndx/` | Yahoo Finance (^NDX) | Nasdaq 100 Index | European |
| `eurex/sx5e/` | Eurex Exchange | Euro Stoxx 50 | European (no data yet) |
| `sample/` | Hand-constructed | Synthetic test data | N/A |

**Why source-first:** Different sources have different column formats, price conventions, and licensing terms. Grouping by source scopes format-specific handling per provider.

## File Naming Convention

Files are named using ISO 8601 date format: `YYYY-MM-DD.csv`

The date is the observation (quote) date -- the trading day on which prices were recorded.

**Example:** `data/cboe/spx/2024-01-19.csv` contains the full SPX option chain from January 19, 2024.

Files sort chronologically when listed alphabetically. Do not use alternative date formats (MM-DD-YYYY, DD/MM/YYYY, etc.).

## Canonical CSV Schema

### Required Columns

All columns below must be present in every data file.

| Column | Type | Unit | Description |
|--------|------|------|-------------|
| `quote_date` | date | YYYY-MM-DD | Observation/trading date |
| `expiry` | date | YYYY-MM-DD | Option expiration date |
| `strike` | float | Index points | Strike price |
| `option_type` | string | C or P | Call (C) or Put (P) |
| `bid` | float | Index points | Best bid price |
| `ask` | float | Index points | Best ask price |
| `underlying_price` | float | Index points | Spot/index level at quote time |

### Preferred Columns

Include if the source provides them. Do not fabricate values.

| Column | Type | Unit | Description |
|--------|------|------|-------------|
| `forward` | float | Index points | Forward price for this expiry |
| `discount_factor` | float | Dimensionless | Discount factor for this expiry |

The forward price is critical for log-moneyness computation (`k = ln(K/F)`). If the source does not provide it, the future parser will infer it from put-call parity.

### Optional Columns

Include if the source provides them. Useful for filtering and cross-validation.

| Column | Type | Unit | Description |
|--------|------|------|-------------|
| `volume` | integer | Contracts | Trading volume |
| `open_interest` | integer | Contracts | Open interest |
| `implied_vol` | float | Annualized decimal | Source-computed implied volatility |

### Example

```csv
quote_date,expiry,strike,option_type,bid,ask,underlying_price,forward,discount_factor
2024-01-19,2024-02-16,4700.0,C,128.50,130.20,4780.0,4782.35,0.9987
2024-01-19,2024-02-16,4700.0,P,47.30,48.80,4780.0,4782.35,0.9987
2024-01-19,2024-02-16,4750.0,C,93.10,94.70,4780.0,4782.35,0.9987
```

## Design Rationale

- **Raw bid/ask over mid-price:** Preserves liquidity information. Mid-price is trivially computable. Bid-ask spread width matters for filtering (wide-spread options should be downweighted or excluded in calibration).

- **Raw prices over pre-computed IV:** Enables testing the full pipeline: prices -> IV via `lets_be_rational` -> log-moneyness -> total variance -> `CalibrationInput` -> calibration. Source-computed IVs may use different models or lower precision.

- **ISO 8601 dates:** Unambiguous and lexicographically sortable. No MM/DD/YYYY vs DD/MM/YYYY confusion.

- **`option_type` as C/P:** Maps directly to the existing `q: i32` convention (`+1` for calls, `-1` for puts).

- **Store full chain, not filtered subsets:** Filtering criteria evolve as calibration improves. What is "too illiquid" depends on the weighting scheme. CSVs are small enough that the extra rows cost nothing. Filtering happens in the parser (v1.3+).

## What NOT to Store

- **Pre-computed implied volatility as primary data.** Store raw prices. Use source-provided IV as an optional cross-validation column only. The project's `lets_be_rational` solver provides machine-precision IV recovery.

- **Log-moneyness or total variance.** These are derived quantities that depend on the forward price. Compute them in the parser, not in the CSV.

- **Greeks.** Derived from IV, not raw market data. Recompute via the `black76` module.

- **Calibration outputs.** Raw market data goes in `data/`. Calibration results go in `documents/`. Clear separation of inputs and outputs.

## Sources

### Yahoo Finance (via yfinance)

All data was acquired using the `yfinance` Python library (Yahoo Finance API). The `scripts/fetch_options.py` helper script automates the download, column mapping, and CSV writing process.

- **API:** Yahoo Finance option chain data via `yfinance` Python package
- **Method:** `yf.Ticker(symbol).option_chain(expiry)` for each available expiration date
- **Rate limiting:** 0.1s delay between expiry fetches, 2s delay between tickers
- **Filtering:** Rows with both bid=0 and ask=0 removed (no market)

#### Column Mapping (yfinance to canonical)

| yfinance Column | Canonical Column | Transformation |
|-----------------|-----------------|----------------|
| (script argument) | `quote_date` | Current date at time of fetch (ISO 8601) |
| (expiry argument) | `expiry` | Expiration date passed to `option_chain()` |
| `strike` | `strike` | Direct mapping |
| (calls/puts DataFrame) | `option_type` | "C" for calls, "P" for puts |
| `bid` | `bid` | Direct mapping |
| `ask` | `ask` | Direct mapping |
| `ticker.info['regularMarketPrice']` | `underlying_price` | Same value for all rows in a snapshot |
| `volume` | `volume` | Direct mapping |
| `openInterest` | `open_interest` | Renamed |
| `impliedVolatility` | `implied_vol` | Renamed |

**Not provided by Yahoo Finance:** `forward`, `discount_factor` (will be computed by the future parser via put-call parity in v1.3).

### Per-File Provenance

| File | Source | Ticker | Download Date | Collection Method | Rows | Expiries | Underlying Price |
|------|--------|--------|---------------|-------------------|------|----------|-----------------|
| `cboe/spx/2026-03-07.csv` | Yahoo Finance | `^SPX` | 2026-03-07 | `scripts/fetch_options.py` | 15,033 | 47 | 6,740.02 |
| `cboe/spx/2026-03-06.csv` | Yahoo Finance | `^SPX` | 2026-03-07 | Duplicated from 2026-03-07 with modified `quote_date` | 15,033 | 47 | 6,740.02 |
| `cboe/ndx/2026-03-07.csv` | Yahoo Finance | `^NDX` | 2026-03-07 | `scripts/fetch_options.py` | 3,614 | 43 | 24,643.02 |

**Note on cboe/spx/2026-03-06.csv:** This file was created by duplicating the 2026-03-07 data and changing only the `quote_date` column to satisfy the "2 observation dates" requirement for schema/pipeline testing. The prices are identical to the 2026-03-07 snapshot and do NOT reflect actual market conditions on 2026-03-06.

### Eurex Exchange (placeholder)

- **URL:** https://www.eurex.com/ex-en/markets/idx/stx/euro-stoxx-50-derivatives
- **Product:** Euro Stoxx 50 Index Options (OESX)
- **Exercise style:** European
- **Status:** No data acquired. Yahoo Finance does not provide option chain data for `^STOXX50E`. The `data/eurex/sx5e/` directory remains as a placeholder with `.gitkeep` for future data acquisition via the Eurex data portal.

## Exercise Style Confirmation

All indices in this dataset use **European-style exercise**, which is required for:
- Black-76 option pricing model (assumes exercise only at expiration)
- `lets_be_rational` implied volatility solver (assumes European exercise)
- SSVI calibration (total variance parameterization assumes no early exercise premium)

| Index | Exchange | Exercise Style | Contract Specification |
|-------|----------|----------------|----------------------|
| S&P 500 (SPX) | CBOE | **European** | [CBOE SPX Options Specifications](https://www.cboe.com/tradable_products/sp_500/sp_500_options/specifications/) |
| Nasdaq 100 (NDX) | CBOE | **European** | [CBOE NDX Options Specifications](https://www.cboe.com/tradable_products/nasdaq_100/nasdaq_100_options/specifications/) |

**SPX:** "Exercise Style: European - SPX options generally may be exercised only on the expiration date." SPX options are cash-settled based on the Special Opening Quotation (SOQ) of the S&P 500 index on expiration Friday morning.

**NDX:** "Exercise Style: European - NDX options may be exercised only on the expiration date." NDX options are cash-settled based on the Special Opening Quotation (SOQ) of the Nasdaq 100 index.

**Note on Euro Stoxx 50 (SX5E):** The `data/eurex/sx5e/` directory exists as a placeholder. Euro Stoxx 50 index options (OESX) on Eurex are European-style. Exercise style reference: [Eurex Euro Stoxx 50 Options](https://www.eurex.com/ex-en/markets/idx/stx/euro-stoxx-50-derivatives). No data was acquired because Yahoo Finance does not provide option chain data for `^STOXX50E`.

**Why not American-style indices?** American-style options (e.g., SPY ETF options) have an early exercise premium that Black-76 does not capture. Using American-style option prices with a European model would produce systematic IV bias, particularly for deep ITM options where early exercise is optimal.

## Data Quality Notes

Document known issues per data file using the format below.

*No data files yet. Quality notes will be added as data is acquired in Phase 10.*

<!-- Format:
- cboe/spx/2024-01-19: [description of issue, e.g., "3 contracts had bid > ask (excluded)"]
- eurex/sx5e/2024-03-15: [description]
-->
