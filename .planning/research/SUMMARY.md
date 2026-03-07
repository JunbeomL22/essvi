# Research Summary: essvi v1.2 Market Data Collection

**Domain:** Option price data storage architecture for SVI/SSVI calibration library
**Researched:** 2026-03-07
**Overall confidence:** HIGH

## Executive Summary

The v1.2 milestone adds a `data/` directory to the essvi project for storing real European-style index option price data. This is a filesystem-only change with no code modifications -- the milestone is data collection, not parsing.

The recommended directory layout is **source-first, then underlying, then date**: `data/{source}/{underlying}/{YYYY-MM-DD}.csv`. This structure groups files by data provider (CBOE, Eurex, JPX), then by instrument (SPX, SX5E, N225), with one CSV per observation date. Each CSV stores raw bid/ask prices, strikes, expiries, underlying price, and (when available) forward price and discount factor. ISO 8601 dates are mandatory in both filenames and column values. A `data/README.md` serves as the data dictionary.

Data files should be git-tracked. The target dataset is 1-3 observation dates totaling under 2 MB -- well within git's comfort zone. These are immutable historical snapshots with no churn. Git LFS is unnecessary. A `data-bulk/` directory can be gitignored later if bulk data is needed.

The critical architectural decision is storing raw prices (bid/ask) rather than pre-computed implied volatilities. This preserves the ability to test the full pipeline (prices -> IV via lets_be_rational -> log-moneyness -> total variance -> CalibrationInput -> calibration). The existing CalibrationInput struct requires no changes; the future parser (v1.3) will produce CalibrationInput from CSV data by computing IV, log-moneyness, and total variance.

## Key Findings

**Stack:** Plain CSV files, UTF-8 encoding, ISO 8601 dates. No new code dependencies for v1.2. Future parser (v1.3) will use `csv` and `chrono` crates.
**Architecture:** `data/{source}/{underlying}/{YYYY-MM-DD}.csv` with `data/README.md` as data dictionary. Git-tracked. Source-first hierarchy because different sources have different column formats and conventions.
**Critical pitfall:** Wrong forward price assumption is the single most impactful error. Store forward in CSV if source provides it; otherwise document that forward must be inferred from put-call parity in the parser.

## Implications for Roadmap

Based on research, suggested phase structure for v1.2:

1. **Phase 1: Define data schema and create directory structure** - Rationale: The column format is the contract that all data files must follow. Define it first.
   - Addresses: data/README.md with column definitions, directory hierarchy creation
   - Avoids: Pitfall of inconsistent formats across files

2. **Phase 2: Acquire first real option chain** - Rationale: With schema defined, download and format one complete option chain snapshot.
   - Addresses: At least 1 day of real data (per PROJECT.md requirements)
   - Avoids: Pitfall of wrong exercise style (target European-only: SPX, Euro Stoxx 50, or Nikkei 225)

3. **Phase 3: Validate and document** - Rationale: Verify data quality, document sourcing, note known issues.
   - Addresses: Data quality notes, sourcing instructions for reproducibility
   - Avoids: Pitfall of undocumented metadata (source, price type, exercise style)

**Phase ordering rationale:**
- Schema first because downloading data without a defined format leads to ad-hoc decisions that are hard to fix later.
- One file first, then validate, because validation may reveal issues with the schema that require adjustment.
- Documentation last because it captures decisions made during phases 1-2.

**Research flags for phases:**
- Phase 2: May need deeper research on which specific data source is accessible for free (Euro Stoxx 50 from Eurex may require paid access; SPX from CBOE free samples may have limited coverage).
- Phase 1 and 3: Standard patterns, unlikely to need additional research.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Plain CSV is the obvious choice. No ambiguity. Future parser crates (csv, chrono) are de facto standards. |
| Features | HIGH | Table stakes and anti-features are clear from the existing CalibrationInput requirements and PROJECT.md scope. |
| Architecture | HIGH | Directory layout follows established patterns from reference SVI calibration projects (mChataign, rfbressan). Integration with existing CalibrationInput is straightforward -- no code changes needed. |
| Pitfalls | HIGH | Forward price, exercise style, and illiquidity filtering are well-documented in academic literature and directly relevant to the SSVI calibration pipeline. |

## Gaps to Address

- **Specific data source availability**: Which free data sources actually provide usable Euro Stoxx 50 or Nikkei 225 option chain CSVs? Eurex and JPX may require accounts or subscriptions. This will be resolved during Phase 2 of implementation.
- **Forward price handling**: Whether the chosen source provides forward/settlement prices or only spot prices will determine how much work the future parser needs. Cannot be resolved until a source is selected.
- **Discount factor source**: If forward is provided directly, discount factor may not be needed. If forward must be computed from put-call parity, a risk-free rate is needed. The specific approach depends on the data source.

---

*Research summary: 2026-03-07*
