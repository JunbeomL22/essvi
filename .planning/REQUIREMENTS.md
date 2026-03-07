# Requirements: essvi v1.2 Market Data Collection

**Defined:** 2026-03-07
**Core Value:** Accurate, arbitrage-free implied volatility surface calibration that handles real-market skew profiles

## v1.2 Requirements

Requirements for v1.2 milestone. Each maps to roadmap phases.

### Data Storage

- [ ] **STOR-01**: `data/` directory created with source-first hierarchy (`data/{source}/{underlying}/`)
- [ ] **STOR-02**: Canonical CSV schema defined (strike, expiry, option_type, bid, ask, underlying_price, + optional forward, discount_factor, volume, open_interest, implied_vol)
- [ ] **STOR-03**: `data/README.md` data dictionary documenting column definitions, units, and file naming conventions

### Data Acquisition

- [ ] **DATA-01**: At least 2 observation dates of real option chain data for one European-style index
- [ ] **DATA-02**: Data from at least 2 different indices (e.g., Nikkei 225 + SPX, or Euro Stoxx 50 + SPX)
- [ ] **DATA-03**: Each snapshot includes both calls and puts across multiple expiry slices

### Documentation

- [ ] **DOCS-01**: Source provenance documented per data file (URL, download date, collection method)
- [ ] **DOCS-02**: Exercise style confirmed as European for each index, with exchange specification references
- [ ] **DOCS-03**: Data quality notes documenting any known issues, excluded rows, or observations

## Future Requirements

### Data Parsing (v1.3+)

- **PARS-01**: CSV parser reads option chain files into typed Rust structs
- **PARS-02**: Forward price extraction via put-call parity
- **PARS-03**: Conversion pipeline from raw prices to CalibrationInput (log-moneyness, total variance)
- **PARS-04**: Illiquidity filtering (bid-ask spread, open interest, volume thresholds)
- **PARS-05**: OTM-only filtering for vol surface construction

### Calibration Integration (v1.4+)

- **INTG-01**: End-to-end pipeline from CSV to SSVI calibration
- **INTG-02**: Vega-weighted calibration using liquidity data

## Out of Scope

| Feature | Reason |
|---------|--------|
| Real-time data feeds | Massively increases complexity; static snapshots sufficient for calibration |
| Automated scraping | Violates exchange ToS; fragile; manual download is sufficient for small datasets |
| Database storage | Overkill for ~5 CSV files; violates pure-Rust spirit |
| Data normalization/parsing | v1.2 is collection only; parsing deferred to v1.3 |
| American-style option data | Black-76 and Let's Be Rational assume European exercise |
| Pre-computed IV as primary data | Store raw prices to test full pipeline |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| STOR-01 | — | Pending |
| STOR-02 | — | Pending |
| STOR-03 | — | Pending |
| DATA-01 | — | Pending |
| DATA-02 | — | Pending |
| DATA-03 | — | Pending |
| DOCS-01 | — | Pending |
| DOCS-02 | — | Pending |
| DOCS-03 | — | Pending |

**Coverage:**
- v1.2 requirements: 9 total
- Mapped to phases: 0
- Unmapped: 9

---
*Requirements defined: 2026-03-07*
*Last updated: 2026-03-07 after initial definition*
