# Phase 9 Research: Data Schema and Directory Structure

**Phase:** 9 — Data Schema and Directory Structure
**Researched:** 2026-03-07
**Confidence:** HIGH

## Research Question

What do I need to know to PLAN this phase well?

Phase 9 creates the data storage foundation: a `data/` directory with source-first hierarchy, a canonical CSV schema, and a `data/README.md` data dictionary. No code changes. No parsing. Pure filesystem and documentation.

## Key Findings

### 1. Directory Layout

**Decision (from milestone research):** Source-first hierarchy: `data/{source}/{underlying}/{YYYY-MM-DD}.csv`

Sources to create directories for:
- `data/cboe/spx/` — CBOE DataShop, S&P 500 Index (reliable free samples)
- `data/eurex/sx5e/` — Eurex, Euro Stoxx 50 (may need account)
- `data/sample/` — Hand-constructed synthetic data for known-answer tests

Rationale: Different sources have different column formats and conventions. Source-first grouping scopes parsing logic per provider. The date filename (`YYYY-MM-DD.csv`) is the atomic unit — one file = one complete option chain snapshot = one SSVI surface calibration input.

### 2. Canonical CSV Schema

**Required columns:**

| Column | Type | Required | Description |
|--------|------|----------|-------------|
| `quote_date` | ISO 8601 date | Yes | Observation date (YYYY-MM-DD) |
| `expiry` | ISO 8601 date | Yes | Option expiration date (YYYY-MM-DD) |
| `strike` | f64 | Yes | Strike price |
| `option_type` | C or P | Yes | Call or Put |
| `bid` | f64 | Yes | Best bid price |
| `ask` | f64 | Yes | Best ask price |
| `underlying_price` | f64 | Yes | Spot/index level at quote time |

**Preferred columns (include if source provides):**

| Column | Type | Description |
|--------|------|-------------|
| `forward` | f64 | Forward price for this expiry |
| `discount_factor` | f64 | Discount factor for this expiry |

**Optional columns (include if source provides, do not fabricate):**

| Column | Type | Description |
|--------|------|-------------|
| `volume` | u64 | Trading volume |
| `open_interest` | u64 | Open interest |
| `implied_vol` | f64 | Source-computed implied volatility |

**Design rationale:**
- Raw bid/ask over mid-price: preserves liquidity information for future weighting
- Raw prices over pre-computed IV: enables testing full pipeline through lets_be_rational
- Forward as preferred: critical for log-moneyness computation `k = ln(K/F)`, but not always available from source
- ISO 8601 dates mandatory: unambiguous, lexicographically sortable

### 3. Data Dictionary (README.md)

The `data/README.md` must document:
1. Column definitions with types, units, and descriptions
2. File naming convention (`YYYY-MM-DD.csv`)
3. Directory hierarchy explanation
4. Source-specific column mappings (e.g., CBOE Optsum column names → canonical names)
5. Data quality conventions (how to note excluded rows, known issues)
6. What NOT to store (no derived quantities like log-moneyness, total variance)

### 4. Git Tracking

Data files should be git-tracked. Target dataset is 1-3 observation dates, under 2 MB total. No Git LFS needed. If bulk data is needed later, use a gitignored `data-bulk/` directory.

### 5. Integration Points

**No code changes in this phase.** The `data/` directory is purely filesystem + documentation.

Future integration (v1.3+):
- `src/data/csv_parser.rs` will read CSVs into `Vec<OptionChainRecord>`
- Pipeline: CSV → mid-price → IV (via lets_be_rational) → log-moneyness → total variance → `CalibrationInput`
- The existing `CalibrationInput` struct requires no modifications

### 6. Requirements Mapping

| Requirement | What Phase 9 Delivers |
|-------------|----------------------|
| STOR-01 | `data/` directory with `data/{source}/{underlying}/` hierarchy |
| STOR-02 | Canonical CSV schema documented (required + optional columns) |
| STOR-03 | `data/README.md` data dictionary with column definitions, units, naming conventions |

All three requirements are fully addressed by this phase.

## Validation Architecture

### What to Validate

Phase 9 produces filesystem structure and documentation, not code. Validation is structural:

1. **Directory existence:** `data/` directory exists at project root
2. **Hierarchy correctness:** At least one source directory exists with an underlying subdirectory (e.g., `data/cboe/spx/`)
3. **Schema documentation:** `data/README.md` contains column definitions table with all 7 required columns
4. **File naming convention:** README documents `YYYY-MM-DD.csv` naming convention
5. **Optional columns documented:** README lists the optional columns (volume, open_interest, implied_vol) and preferred columns (forward, discount_factor)

### Validation Methods

| Check | Method | Pass Criteria |
|-------|--------|---------------|
| Directory exists | `test -d data/` | Directory present |
| Source hierarchy | `ls data/*/` | At least one source with underlying subdirectory |
| README exists | `test -f data/README.md` | File present and non-empty |
| Required columns documented | `grep` in README | All 7 required columns named |
| Naming convention | `grep` in README | `YYYY-MM-DD` pattern mentioned |

### What NOT to Validate

- CSV file contents (no data files exist yet — that is Phase 10)
- Code compilation (no code changes in this phase)
- Parser functionality (future phase)

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Schema needs revision after seeing real data | Medium | Low | Schema is documentation; easy to update in Phase 10 |
| Source column mappings unknown until download | Expected | None for Phase 9 | Document placeholder mappings; finalize in Phase 10 |
| Forward price handling unclear | Medium | Low | Document both approaches (source-provided vs put-call parity) in README |

## Complexity Assessment

**LOW.** This phase creates directories and writes one markdown file. No code, no dependencies, no compilation. Estimated effort: 1 plan, 1 wave, completable in a single session.

## RESEARCH COMPLETE

---

*Phase 9 research: 2026-03-07*
