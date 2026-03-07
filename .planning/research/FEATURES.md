# Feature Landscape

**Domain:** Option market data collection for SVI/SSVI calibration
**Researched:** 2026-03-07

## Table Stakes

Features users expect. Missing = product feels incomplete.

### Data Fields

The existing calibration pipeline (`CalibrationInput`) requires log-moneyness `k = ln(K/F)` and total variance `w = T * sigma_iv^2`. The IV solver (`lets_be_rational`) and pricer (`black76`) require forward price, strike, time-to-expiry, option type, and option price. Every field below is needed to derive these downstream inputs.

| Field | Why Expected | Complexity | Notes |
|-------|--------------|------------|-------|
| **Strike price (K)** | Needed for log-moneyness k = ln(K/F) and all pricing functions | Low | Raw field from any exchange data source |
| **Expiry date** | Needed to compute time-to-expiry T in years | Low | Convert to T using business-day or ACT/365 convention |
| **Option type (call/put)** | Maps to q = +1/-1 in Black-76 and Let's Be Rational | Low | Essential for put-call parity forward extraction |
| **Settlement price** | Primary option price for calibration; end-of-day, exchange-determined, no bid-ask ambiguity | Low | Eurex computes via Black-76 model; JPX uses SQ method; CBOE uses marking prices |
| **Underlying index level** | Needed as reference for forward price derivation | Low | Spot index value at settlement time (e.g., SX5E close, N225 close, SPX close) |
| **Exercise style (European/American)** | Determines whether Black-76 / Let's Be Rational apply directly or need de-Americanization | Low | Store as metadata; critical for downstream IV computation correctness |
| **Observation date** | Identifies when snapshot was taken | Low | Required for T calculation: T = (expiry - obs_date) / 365 |

### Data Organization

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **`data/` directory with clear naming** | Callers must locate the right data files without guessing | Low | Convention: `data/{index}/{YYYY-MM-DD}.csv` or similar |
| **At least one complete option chain snapshot** | One day of data per index = minimum for validating the pipeline end-to-end | Low | Single trading day with all listed strikes and expiries |
| **Multiple expiry slices in one snapshot** | SSVI calibrates across the surface; single-expiry data is insufficient for surface fitting | Med | Must include short-dated (< 1 month) through long-dated (> 1 year) expiries |
| **Both calls and puts per strike** | Put-call parity requires matched call/put pairs to extract the implied forward price | Low | OTM-only filtering happens downstream, not at collection |

### Derived / Companion Fields

These fields are not raw exchange data but are essential companions that must either be stored alongside or be unambiguously derivable from the raw data.

| Field | Why Expected | Complexity | Notes |
|-------|--------------|------------|-------|
| **Forward price (F)** | The most critical derived field; required for k = ln(K/F) | Med | Extracted via put-call parity: C - P = df * (F - K). Requires matched call/put pairs at multiple strikes. Alternatively, use listed futures price if available. |
| **Risk-free rate / discount factor** | Needed for discounting and for forward extraction via put-call parity | Med | For European indices: ECB deposit rate (Euro Stoxx 50), BOJ rate (Nikkei 225), Fed Funds/SOFR (SPX). Can also be implied from put-call parity. |
| **Time to expiry (T)** | Direct input to Black-76 and total variance w = T * sigma^2 | Low | Computed from observation date and expiry date. Convention matters: ACT/365 is standard for vol surface work. |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Bid/ask prices** | Enables mid-price computation, spread-based liquidity filtering, and bid/ask IV calculation for tighter calibration bounds | Med | Not all free sources provide bid/ask (settlement-only is common for end-of-day exchange data). Yahoo Finance provides bid/ask but with caveats (delayed, unstable API). |
| **Volume and open interest** | Enables liquidity-weighted calibration (more weight on liquid strikes); standard quality filter in academic and production pipelines | Low | Available from most exchange sources. Useful for the `weights` field in `CalibrationInput`. |
| **Market-quoted implied volatility** | Provides a cross-check for the IV solver output; some exchanges (Eurex) publish settlement IV alongside settlement price | Low | Useful for validation but should not replace computing IV from prices (which tests the full pipeline). |
| **Multiple observation dates** | Enables time-series analysis of parameter stability, term structure dynamics, and calendar arbitrage testing across dates | Med | v1.2 scope is "at least 1 day"; multiple days adds value but increases collection effort linearly. |
| **OTM-only pre-filtering** | Standard practice: use OTM puts for k < 0, OTM calls for k > 0 to avoid dual pricing and maximize liquidity | Med | This is a data processing concern, not a collection concern. Better handled in a future parsing milestone. Should be documented but not implemented at collection time. |
| **Greeks from exchange** | Delta, gamma, vega, theta as published by exchange; useful for cross-validation of Black-76 greeks module | Low | Nice for testing but not needed for calibration. |
| **Futures price for the underlying** | Direct observable for forward price F, avoiding put-call parity extraction entirely | Med | Euro Stoxx 50 futures (FESX) and Nikkei 225 futures trade on same exchanges. SPX has E-mini futures on CME. Greatly simplifies forward determination. |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Real-time data feed / API integration** | Massively increases complexity (websockets, authentication, rate limiting, data normalization). Out of scope for a calibration library. | Collect end-of-day snapshots manually and store as static files. |
| **Automated data scraping** | Violates terms of service for most exchanges (CBOE explicitly blocks auto-extraction). Fragile, maintenance-heavy. | Manual download from official exchange sources; document the procedure. |
| **Database storage (SQLite, Postgres, etc.)** | Adds external dependency to a zero-dependency Rust library. Overkill for static reference data. | Use flat CSV/JSON files in `data/`. Pure Rust parsing in a future milestone. |
| **Data normalization at collection time** | Mixing raw and derived data creates ambiguity about what came from the exchange vs. what was computed. | Store raw exchange data as-is. Normalization (log-moneyness, total variance conversion) belongs in the parsing/pipeline milestone. |
| **American option price adjustment (de-Americanization)** | Complex (requires American option pricing model, binomial trees or PDE solvers). The library uses Black-76 which assumes European exercise. | Prefer European-style options (Euro Stoxx 50, Nikkei 225, SPX). All three are European exercise. Avoid collecting American-style option data (e.g., SPY options) entirely. |
| **Dividend/yield curve data** | Adds significant complexity and external data dependencies for what is a secondary concern | Forward price from put-call parity or listed futures implicitly accounts for dividends and rates. |
| **Intraday tick data** | Orders of magnitude more data, storage, and parsing complexity for negligible calibration benefit | End-of-day settlement prices are the industry standard for vol surface calibration. |

## European vs. American Exercise Style

This distinction is critical for the downstream IV computation pipeline.

### Why It Matters

The existing `lets_be_rational` IV solver and `black76` pricer assume **European exercise**. Black-76 is the standard model for European-style futures/index options. Applying these models to American-style options introduces systematic error because:

1. **Early exercise premium**: American options are worth at least as much as European options. The difference (early exercise premium) inflates the price, and computing IV with Black-76 from an American price gives an upward-biased IV.

2. **Put-call parity breaks**: Put-call parity (C - P = df * (F - K)) holds exactly only for European options. For American options, it becomes an inequality, making forward extraction from put-call parity unreliable.

3. **No de-Americanization in the library**: Converting American prices to European-equivalent prices requires a separate American pricing model (binomial tree, Barone-Adesi-Whaley, etc.), which the library does not have and should not need.

### Target Instruments Are All European

| Index | Exchange | Exercise Style | Settlement | Confidence |
|-------|----------|---------------|------------|------------|
| Euro Stoxx 50 (SX5E) | Eurex | European | Cash-settled, Black-76 based daily settlement | HIGH (verified: Eurex product specs) |
| Nikkei 225 | Osaka Exchange (JPX) | European | Cash-settled, SQ-based final settlement | HIGH (verified: JPX contract specs state "European. The option may be exercised only at its expiration.") |
| SPX (S&P 500 Index) | CBOE | European | Cash-settled, AM or PM settlement | HIGH (verified: CBOE product specs, SPX is European-style despite common misconception) |

All three target indices use European-style options. This is by design: major index options are almost universally European because cash settlement eliminates early exercise incentives that exist for single-stock options.

**Key implication**: No de-Americanization logic is needed. The existing Black-76 pricer and Let's Be Rational IV solver apply directly.

### What to Avoid

- **SPY options** (American-style ETF options on the S&P 500 ETF) -- these are NOT SPX options
- **Single-stock options** (almost all are American-style in the US)
- **Any option without explicit European exercise confirmation**

The `exercise_style` field should be stored as metadata to make this distinction explicit and prevent accidental use of American-style data.

## Feature Dependencies

```
Observation date + Expiry date --> Time to expiry (T)
Strike (K) + Forward price (F) --> Log-moneyness (k = ln(K/F))
Matched call/put settlement prices + Strike + Discount factor --> Forward price (F) via put-call parity
    OR
Listed futures price --> Forward price (F) directly
Implied volatility (sigma) + T --> Total variance (w = T * sigma^2)
    OR
Option price + F + K + T + q --> sigma via Let's Be Rational --> w = T * sigma^2

Settlement price + F + K + T + q --> CalibrationInput (via IV solver)
    requires: forward price (from put-call parity or futures)
    requires: time to expiry (from dates)
    requires: exercise style = European (for Black-76 validity)
```

The dependency chain shows that forward price extraction is the critical intermediate step. Without a reliable forward, log-moneyness cannot be computed, and the entire calibration pipeline cannot run. This is why storing either matched call/put pairs (for put-call parity) or listed futures prices is table-stakes.

## MVP Recommendation

Prioritize:
1. **One complete snapshot of Nikkei 225 options from JPX** -- free CSV download with settlement prices, European-style confirmed, well-structured exchange data
2. **Store raw exchange data in `data/nikkei225/` directory** with clear naming
3. **Include metadata file** documenting: observation date, underlying level, data source URL, exercise style, settlement methodology
4. **If JPX data is insufficient, use Euro Stoxx 50 from Eurex** as secondary source (may require Eurex DataShop access)
5. **SPX as last-resort fallback** -- CBOE data requires commercial license for bulk download

Defer:
- **Multiple observation dates**: One day is sufficient for v1.2 validation
- **Bid/ask prices**: Settlement prices are cleaner for initial calibration validation
- **OTM filtering, forward extraction, log-moneyness conversion**: These are parsing/pipeline features for a future milestone
- **Volume/open interest weighting**: Adds value but is not needed for basic calibration validation

## Sources

- [Eurex Euro Stoxx 50 Index Options Specs](https://www.eurex.com/ex-en/markets/idx/stx/euro-stoxx-50-derivatives/products/EURO-STOXX-50-Index-Options-46548) -- European exercise, Black-76 settlement
- [JPX Nikkei 225 Options Contract Specs](https://www.jpx.co.jp/english/derivatives/products/domestic/225options/01.html) -- European exercise confirmed
- [JPX Settlement Prices Download](https://www.jpx.co.jp/english/markets/derivatives/settlement-price/index.html) -- Free CSV with daily settlement prices
- [CBOE SPX Options Specs](https://www.cboe.com/tradable_products/sp_500/spx_options/specifications/) -- European exercise, cash-settled
- [Eurex Historical Data](https://www.eurex.com/ex-en/data/historical-data) -- via Deutsche Boerse Data Shop (likely paid)
- [CBOE DataShop](https://datashop.cboe.com/) -- Commercial; SPX data requires CGI license ($1k+/month)
- [historicaloptiondata.com File Structures](https://historicaloptiondata.com/historical-options-data-file-structures/) -- Reference for standard option CSV field conventions
- [Gatheral & Jacquier, "Arbitrage-free SVI volatility surfaces"](https://arxiv.org/pdf/1204.0646) -- SVI/SSVI calibration data requirements, OTM filtering recommendation
- [SVI Calibration Tutorial (KTH thesis)](https://kth.diva-portal.org/smash/get/diva2:744907/FULLTEXT02.pdf) -- Log-moneyness and total variance definitions, calibration data flow
