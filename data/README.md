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
| `cboe/spx/` | CBOE DataShop | S&P 500 Index | European |
| `eurex/sx5e/` | Eurex Exchange | Euro Stoxx 50 | European |
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

### CBOE DataShop

- **URL:** https://datashop.cboe.com/end-of-day-options-summary
- **Product:** Optsum (End-of-Day Options Summary)
- **Availability:** Free sample / paid subscription
- **Column mapping:** To be documented when data is acquired (Phase 10)

### Eurex Exchange

- **URL:** https://www.eurex.com/ex-en/markets/idx/stx/euro-stoxx-50-derivatives
- **Product:** Euro Stoxx 50 Index Options (OESX)
- **Exercise style:** European
- **Column mapping:** To be documented when data is acquired (Phase 10)

## Data Quality Notes

Document known issues per data file using the format below.

*No data files yet. Quality notes will be added as data is acquired in Phase 10.*

<!-- Format:
- cboe/spx/2024-01-19: [description of issue, e.g., "3 contracts had bid > ask (excluded)"]
- eurex/sx5e/2024-03-15: [description]
-->
