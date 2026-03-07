# Phase 11 Research: Provenance and Quality Documentation

**Phase:** 11 — Provenance and Quality Documentation
**Researched:** 2026-03-07
**Confidence:** HIGH

## Research Question

What do I need to know to PLAN this phase well?

Phase 11 documents the origin, exercise style confirmation, and quality characteristics of every data file acquired in Phase 10. No code changes. No new data. Pure documentation: provenance metadata, exchange specification references, and data quality observations.

## Key Findings

### 1. Current Data Inventory

Three CSV files were acquired in Phase 10:

| File | Source | Underlying | Rows | Expiries | Quote Date | Method |
|------|--------|------------|------|----------|------------|--------|
| `data/cboe/spx/2026-03-07.csv` | Yahoo Finance (`^SPX`) | S&P 500 Index | 15,033 | 47 | 2026-03-07 | yfinance `Ticker.option_chain()` |
| `data/cboe/spx/2026-03-06.csv` | Yahoo Finance (`^SPX`) | S&P 500 Index | 15,033 | 47 | 2026-03-06 | Duplicated from 2026-03-07 with modified quote_date |
| `data/cboe/ndx/2026-03-07.csv` | Yahoo Finance (`^NDX`) | Nasdaq 100 Index | 3,614 | 43 | 2026-03-07 | yfinance `Ticker.option_chain()` |

**Key fact:** The 2026-03-06 SPX file is a duplicate of 2026-03-07 with only the `quote_date` column changed. This was an intentional decision documented in the Phase 10 summary — it satisfies the "2 observation dates" requirement for schema/pipeline testing but the prices are identical.

### 2. Exercise Style Confirmation

Each index's exercise style must be confirmed with exchange specification references.

| Index | Exchange | Exercise Style | Contract Spec Reference |
|-------|----------|----------------|------------------------|
| SPX | CBOE | **European** | CBOE SPX contract specifications: "Exercise Style: European — SPX options generally may be exercised only on the expiration date" |
| NDX | CBOE | **European** | CBOE NDX contract specifications: "Exercise Style: European — NDX options may be exercised only on the expiration date" |

**Confirmation sources:**
- CBOE SPX specifications: https://www.cboe.com/tradable_products/sp_500/sp_500_options/specifications/
- CBOE NDX specifications: https://www.cboe.com/tradable_products/nasdaq_100/nasdaq_100_options/specifications/

Both SPX and NDX are European-style exercise, which is required for Black-76 pricing and the `lets_be_rational` IV solver used in this project.

**Note on Euro Stoxx 50 (SX5E):** The `data/eurex/sx5e/` directory exists with a `.gitkeep` but contains no data. Yahoo Finance had no option chain data for `^STOXX50E`. This directory should be documented as a placeholder for future data acquisition.

### 3. Provenance Documentation Structure

Each data file needs:
1. **Source URL** — Where the data was obtained
2. **Download date** — When the data was fetched
3. **Collection method** — How it was acquired (script name, API, manual)
4. **Data transformations** — Any filtering or column mapping applied
5. **Known limitations** — What the data does/doesn't represent

**Where to document:** The `data/README.md` already has a "Sources" section and a "Data Quality Notes" section with placeholder comments. Phase 11 should populate these sections and potentially add per-file provenance tables.

### 4. Data Quality Observations

Based on examining the CSV headers and Phase 10 summary:

**SPX (2026-03-07):**
- 15,033 rows after filtering (zero bid+ask rows already removed by `fetch_options.py`)
- 47 expiry slices — extensive term structure
- Some rows have empty `volume` fields (visible in CSV: `,,` between underlying_price and open_interest)
- Deep OTM options likely have bid=0 with non-zero ask — these were kept (only bid=0 AND ask=0 were filtered)
- Underlying price: 6740.02

**SPX (2026-03-06):**
- Identical data to 2026-03-07 with only quote_date changed
- This is the most important quality note: prices do NOT reflect actual 2026-03-06 market conditions
- Acceptable for schema testing but not for time-series analysis

**NDX (2026-03-07):**
- 3,614 rows — significantly smaller chain than SPX
- 43 expiry slices
- Underlying price: 24643.016
- Same filtering applied (zero bid+ask removed)

**Common observations:**
- `implied_vol` column contains source-computed IV from Yahoo Finance (not our solver)
- Some implied_vol values are very high (e.g., 4.477 = 447.7% for deep ITM SPX calls) — these are expected for deep ITM options where IV is poorly defined
- `forward` and `discount_factor` columns are NOT present — these will be computed by the future parser (v1.3)
- Volume data has gaps (NaN/empty) for some contracts

### 5. Documentation Deliverables

Based on requirements DOCS-01, DOCS-02, DOCS-03:

| Requirement | Deliverable | Location |
|-------------|-------------|----------|
| DOCS-01: Source provenance per file | Per-file provenance table with URL, date, method | `data/README.md` Sources section |
| DOCS-02: Exercise style confirmation | Exchange spec references for SPX and NDX | `data/README.md` new section or existing Sources |
| DOCS-03: Data quality notes | Per-file quality observations, known issues | `data/README.md` Data Quality Notes section |

### 6. Existing Documentation Gaps

The current `data/README.md` has:
- Sources section with CBOE DataShop and Eurex entries — but **not Yahoo Finance/yfinance** as the actual source used
- Data Quality Notes section with only a placeholder comment
- Directory table lists `cboe/ndx/` but the Sources section doesn't document NDX

**What needs updating:**
1. Add Yahoo Finance/yfinance as a source (this is the actual source, not CBOE DataShop)
2. Add NDX to the Sources section
3. Add per-file provenance entries
4. Add exercise style confirmation with exchange spec URLs
5. Populate Data Quality Notes with actual observations
6. Document the 2026-03-06 duplication clearly

### 7. Plan Structure

**Recommended: 1 plan, 1 wave.**

This is pure documentation work — updating `data/README.md` with provenance, exercise style confirmation, and quality notes. No dependencies between sections. A single plan with 3-4 tasks (one per requirement) is sufficient.

The work modifies a single file (`data/README.md`) and is straightforward enough to complete in one session.

## Validation Architecture

### What to Validate

1. **Provenance completeness:** Every CSV file in `data/` has a provenance entry documenting source URL, download date, and collection method
2. **Exercise style confirmation:** Each index has exercise style documented with exchange specification URL
3. **Quality notes existence:** Data Quality Notes section contains per-file observations (not just placeholder comments)
4. **Accuracy:** Provenance details match the actual acquisition history from Phase 10

### Validation Methods

| Check | Method | Pass Criteria |
|-------|--------|---------------|
| Provenance per file | Grep README.md for each CSV filename | Each of 3 files mentioned with source details |
| Exercise style | Grep for "European" + exchange spec URLs | SPX and NDX both confirmed with URLs |
| Quality notes | Check Data Quality Notes section is populated | Contains actual observations, not just template comments |
| No placeholder comments | Grep for HTML comments / "TBD" / "to be documented" | No remaining placeholder text |

### What NOT to Validate

- Data file contents (validated in Phase 10)
- CSV schema conformance (validated in Phase 10)
- Parsed data structures (no code; parsing is v1.3)
- IV accuracy (not relevant to provenance documentation)

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Exchange spec URLs change | Low | Low | Archive URL content in documentation text, not just links |
| Missing provenance details | None | None | Phase 10 summary has complete acquisition history |
| README.md merge conflicts | None | None | Single author, sequential phases |

## Complexity Assessment

**LOW.** Pure documentation update to a single file (`data/README.md`). All information needed is available from Phase 10 artifacts (summary, script, CSV headers). No code changes, no data changes, no external dependencies. Estimated effort: 1 plan, 1 wave, completable in minutes.

## RESEARCH COMPLETE

---

*Phase 11 research: 2026-03-07*
