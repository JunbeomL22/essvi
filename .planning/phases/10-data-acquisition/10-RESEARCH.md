# Phase 10 Research: Data Acquisition

**Phase:** 10 — Data Acquisition
**Researched:** 2026-03-07
**Confidence:** HIGH

## Research Question

What do I need to know to PLAN this phase well?

Phase 10 acquires real European-style index option chain data and stores it in the canonical CSV schema defined in Phase 9. No code changes. No parsing. Pure data download, format conversion, and file placement.

## Key Findings

### 1. Data Sources and Availability

**Target indices (European-style exercise):**

| Index | Ticker | Exchange | Exercise Style | Source |
|-------|--------|----------|----------------|--------|
| S&P 500 | SPX | CBOE | European | Yahoo Finance (^SPX) |
| Euro Stoxx 50 | SX5E | Eurex | European | Yahoo Finance (^STOXX50E) |
| Nikkei 225 | NKY | Osaka Exchange (JPX) | European | Yahoo Finance (^N225) |
| S&P 100 (European) | XEO | CBOE | European | Yahoo Finance (^XEO) |

**Primary acquisition method: Yahoo Finance via yfinance Python library.**

Yahoo Finance provides free, real-time option chain data for all major indices including ^SPX, ^STOXX50E, and ^N225. The `yfinance` library (`pip install yfinance`) provides programmatic access:

```python
import yfinance as yf
ticker = yf.Ticker("^SPX")
expirations = ticker.options  # list of expiry dates
chain = ticker.option_chain("2024-03-15")  # returns calls + puts DataFrames
```

**yfinance option chain columns:**
- `contractSymbol`, `lastTradeDate`, `strike`, `lastPrice`, `bid`, `ask`
- `change`, `percentChange`, `volume`, `openInterest`, `impliedVolatility`
- `inTheMoney`, `contractSize`, `currency`

This provides all 7 required canonical columns (after mapping) plus optional columns (volume, open_interest, implied_vol).

**Alternative sources (fallback):**
- CBOE DataShop Optsum: End-of-day option summary for SPX. Paid subscription for current data; historical 2005-2019 available. Contains volume, open interest, OHLC prices but may lack bid/ask.
- Eurex historical data portal: Registration may be required.
- Barchart.com: Shows option chains for Euro Stoxx 50 futures options.

**Recommendation:** Use yfinance for all indices. It is free, provides bid/ask data, and covers SPX, Euro Stoxx 50, and Nikkei 225. Data is real-time snapshots (15-min delay during market hours), which is perfectly suitable for calibration testing.

### 2. Column Mapping: yfinance to Canonical Schema

| yfinance Column | Canonical Column | Transformation |
|-----------------|-----------------|----------------|
| (derived from expiry arg) | `quote_date` | Use current date (the observation date) |
| (derived from expiry arg) | `expiry` | The expiration date passed to `option_chain()` |
| `strike` | `strike` | Direct mapping |
| (derived from calls/puts) | `option_type` | "C" for calls DataFrame, "P" for puts DataFrame |
| `bid` | `bid` | Direct mapping |
| `ask` | `ask` | Direct mapping |
| (from ticker.info) | `underlying_price` | `ticker.info['regularMarketPrice']` or `ticker.fast_info['last_price']` |
| N/A | `forward` | Not provided by yfinance (inferred from put-call parity in v1.3 parser) |
| N/A | `discount_factor` | Not provided by yfinance |
| `volume` | `volume` | Direct mapping |
| `openInterest` | `open_interest` | Direct mapping (rename) |
| `impliedVolatility` | `implied_vol` | Direct mapping (rename) |

### 3. Acquisition Script Strategy

A simple Python script (`scripts/fetch_options.py`) can:
1. Accept ticker symbol, output directory, and optional date as arguments
2. Fetch all available expiration dates
3. For each expiry, download the full option chain (calls + puts)
4. Map columns to canonical schema
5. Write a single CSV per observation date: `data/{source}/{underlying}/{YYYY-MM-DD}.csv`

**Key considerations:**
- yfinance provides data per expiry. A single CSV should contain ALL expiries for one observation date (many expiry slices in one file).
- The script should be a helper tool, not production code. It lives in `scripts/` not `src/`.
- Filter out rows where bid = 0 AND ask = 0 (no market, useless for calibration).
- Yahoo Finance data is available during market hours with ~15 min delay. Run during US trading hours for best data quality.
- Rate limiting: yfinance can be rate-limited. Add small delays between requests.

### 4. Data Volume and Storage

**Estimated data per observation date:**
- SPX: ~20-30 expiry dates x ~200 strikes x 2 (C+P) = ~8,000-12,000 rows per observation date
- Euro Stoxx 50: ~10-15 expiry dates x ~100 strikes x 2 = ~2,000-3,000 rows
- Nikkei 225: ~8-12 expiry dates x ~80 strikes x 2 = ~1,280-1,920 rows

**Per CSV:** ~200-500 KB uncompressed. Total for 4-6 observation dates across 2 indices: ~2-4 MB. Well within git tracking limits (no LFS needed).

### 5. Success Criteria Mapping

| Criterion | How Phase 10 Delivers |
|-----------|----------------------|
| At least 2 observation dates for one index | Run script on 2 different trading days for SPX |
| Data from at least 2 different European-style indices | Fetch SPX + one of: Euro Stoxx 50, Nikkei 225, XEO |
| Each snapshot has calls and puts across multiple expiry slices | yfinance returns all available expiries; script collects all |
| All CSVs conform to canonical schema | Script maps columns to Phase 9 schema before writing |

### 6. Requirements Mapping

| Requirement | What Phase 10 Delivers |
|-------------|----------------------|
| DATA-01 | At least 2 observation dates of real option chain data for SPX |
| DATA-02 | Data from at least 2 different European-style indices (SPX + Euro Stoxx 50 or Nikkei 225) |
| DATA-03 | Each snapshot includes both calls and puts across multiple expiry slices |

### 7. Plan Structure

**Recommended: 2 plans, 2 waves.**

- **Plan 10-01 (Wave 1):** Create acquisition script (`scripts/fetch_options.py`) and fetch first dataset (SPX, 1 observation date). This validates the script works and data conforms to schema.
- **Plan 10-02 (Wave 2):** Use script to fetch remaining data: second SPX observation date + second index (Euro Stoxx 50 or Nikkei 225). Depends on Plan 01 to validate the script.

Alternative: Single plan if the script is simple enough and both indices can be fetched in one session. But 2 plans provides a validation checkpoint after the first dataset.

**Revised recommendation: 1 plan, 1 wave.**

The script is straightforward (< 100 lines), and fetching data is fast. Splitting into 2 plans adds overhead without benefit since the script can be validated by running it once and checking the output. A single plan that creates the script and uses it to fetch all required data is sufficient.

## Validation Architecture

### What to Validate

1. **CSV file existence:** At least 2 CSV files exist for one index (2 observation dates)
2. **Multi-index coverage:** CSV files exist for at least 2 different indices
3. **Schema conformance:** Every CSV has all 7 required columns with correct names
4. **Call/Put coverage:** Each CSV contains both option_type "C" and "P" rows
5. **Multi-expiry coverage:** Each CSV contains at least 3 distinct expiry dates
6. **Data quality:** No CSV has only zero-bid/zero-ask rows

### Validation Methods

| Check | Method | Pass Criteria |
|-------|--------|---------------|
| File existence (SPX) | `ls data/*/spx/*.csv \| wc -l` | >= 2 files |
| Multi-index | `ls data/*/ \| sort -u` | >= 2 directories with CSV files |
| Schema columns | `head -1` of each CSV | Contains all 7 required column names |
| Call+Put | `cut -d, -f4 \| sort -u` | Contains both "C" and "P" |
| Multi-expiry | `cut -d, -f2 \| sort -u \| wc -l` | >= 3 unique expiry dates per file |
| Non-trivial data | `wc -l` | >= 100 rows per file |

### What NOT to Validate

- Parsed data structures (no code changes; parsing is v1.3)
- Forward price computation (derived quantity, computed by future parser)
- IV accuracy (yfinance provides source-computed IV as optional column)

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| yfinance rate limiting | Medium | Low | Add delays between requests; retry with backoff |
| Yahoo Finance data unavailable for ^STOXX50E or ^N225 | Low-Medium | Medium | Fall back to ^XEO (S&P 100 European) as second index |
| Bid/ask = 0 for deep OTM options | Expected | None | Filter in script; document in quality notes |
| Weekend/holiday: no live data | Expected if run off-hours | None | yfinance returns last available data; quote_date reflects actual observation |
| yfinance API changes | Low | Medium | Pin yfinance version in script; document manual CSV creation as fallback |

## Complexity Assessment

**LOW-MEDIUM.** The Python script is simple (<100 lines). The main variable is data availability from Yahoo Finance for non-US indices. Estimated effort: 1 plan, 1 wave, completable in a single session. The script is a tooling artifact, not production code.

## RESEARCH COMPLETE

---

*Phase 10 research: 2026-03-07*
