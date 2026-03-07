# Architecture Patterns

**Domain:** Option price data storage for SVI/SSVI calibration library
**Researched:** 2026-03-07

## Recommended Architecture

### Overview

Add a `data/` directory at the project root to store real option price data files. The directory uses a flat, source-first hierarchy with one CSV per observation date per underlying. Data files are git-tracked (small, curated datasets -- not bulk market feeds). A future `src/data/` module will parse these files into the existing `CalibrationInput` pipeline.

```
essvi/
├── data/                          # NEW: Market data storage
│   ├── README.md                  # Data dictionary, sourcing notes
│   ├── cboe/                      # Source: CBOE DataShop
│   │   └── spx/                   # Underlying: S&P 500 Index
│   │       └── 2024-01-19.csv     # One trading date
│   ├── eurex/                     # Source: Eurex exchange
│   │   └── sx5e/                  # Underlying: Euro Stoxx 50
│   │       └── 2024-03-15.csv     # One trading date
│   └── sample/                    # Hand-constructed validation data
│       └── synthetic_3slice.csv   # Known-answer test data
├── src/
│   ├── data/                      # FUTURE: Parser module (v1.3+)
│   │   ├── mod.rs
│   │   └── csv_parser.rs
│   ├── lib.rs                     # Add `pub mod data;` when parser exists
│   └── ...existing modules...
├── tests/
│   └── data_loading.rs            # FUTURE: Integration tests for parsing
└── ...existing files...
```

### Why This Layout

**Source-first, then underlying, then date** because:

1. Different sources have different column formats, licensing terms, and reliability. Grouping by source keeps format-specific parsing logic scoped.
2. Within a source, data is always for a specific underlying. The underlying determines forward price calculation conventions and option exercise style.
3. The date is the leaf because a single date's option chain is the natural unit of work for SSVI calibration (one snapshot produces all expiry slices).

**Rejected alternative: date-first layout** (`data/2024-01-19/cboe/spx.csv`). This fragments sources across date directories, making it harder to add a new source or see all data from one provider at a glance. SVI calibration processes one underlying at a time across its expiries, not cross-underlying on the same date.

**Rejected alternative: flat layout** (`data/spx_2024-01-19.csv`). Mixes sources with no structure. Falls apart once you have data from multiple providers for the same underlying (e.g., CBOE vs broker snapshot).

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `data/` directory (filesystem) | Store raw CSV files as-is from source | Human (manual download), future parser |
| `data/README.md` | Document column meanings, sourcing process, licensing | Human reference |
| `data/{source}/{underlying}/{date}.csv` | One observation date of option chain data | Future `src/data/csv_parser.rs` |
| `data/sample/` | Store hand-crafted CSVs with known-answer calibration results | Tests, validation |
| Future `src/data/mod.rs` | Define `OptionChainRecord` struct, load/parse API | `src/calibration.rs` via `CalibrationInput` |

### Data Flow

**Current (v1.2 -- collection only):**
```
Manual download from exchange website
  -> Save as CSV in data/{source}/{underlying}/{date}.csv
  -> Human inspects data quality
  -> No code integration yet
```

**Future (v1.3+ -- parsing integration):**
```
data/{source}/{underlying}/{date}.csv
  -> src/data/csv_parser.rs reads CSV, produces Vec<OptionChainRecord>
  -> Conversion logic groups by expiry, computes log-moneyness k = ln(K/F)
  -> Computes total variance w = iv^2 * T (or derives IV from prices via lets_be_rational)
  -> Produces Vec<CalibrationInput> for each expiry slice
  -> Feeds into existing calibrate() / calibrate_with_calendar_penalty()
```

## CSV File Format

### Recommended Columns

The CSV should preserve the raw data as close to source format as possible, with a minimal set of derived columns that are unambiguous. Use this canonical column set.

```csv
quote_date,expiry,strike,option_type,bid,ask,underlying_price,forward,discount_factor
2024-01-19,2024-02-16,4700.0,C,128.50,130.20,4780.0,4782.35,0.9987
2024-01-19,2024-02-16,4700.0,P,47.30,48.80,4780.0,4782.35,0.9987
2024-01-19,2024-02-16,4750.0,C,93.10,94.70,4780.0,4782.35,0.9987
```

| Column | Type | Required | Description |
|--------|------|----------|-------------|
| `quote_date` | ISO 8601 date | Yes | Observation date (YYYY-MM-DD) |
| `expiry` | ISO 8601 date | Yes | Option expiration date (YYYY-MM-DD) |
| `strike` | f64 | Yes | Strike price |
| `option_type` | C or P | Yes | Call or Put |
| `bid` | f64 | Yes | Best bid price |
| `ask` | f64 | Yes | Best ask price |
| `underlying_price` | f64 | Yes | Spot/index level at quote time |
| `forward` | f64 | Preferred | Forward price for this expiry (if available from source) |
| `discount_factor` | f64 | Preferred | Discount factor for this expiry (if available) |

**Optional columns** (include if source provides them, but do not fabricate):

| Column | Type | Description |
|--------|------|-------------|
| `volume` | u64 | Trading volume |
| `open_interest` | u64 | Open interest |
| `implied_vol` | f64 | Source-computed implied volatility |

### Why These Columns

- **bid/ask rather than mid or last**: Mid-price is trivially computable. Keeping bid/ask preserves information about liquidity and spread width, which matters for filtering (wide-spread options should be downweighted or excluded in calibration).
- **forward and discount_factor**: The SSVI calibration works in log-moneyness space `k = ln(K/F)` where F is the forward. Getting the forward right is critical. If the source provides it (CBOE, Eurex settlement data), store it. If not, it can be inferred from put-call parity in the future parser.
- **ISO 8601 dates**: Unambiguous, sortable, standard. No MM/DD/YYYY vs DD/MM/YYYY confusion.
- **option_type as C/P**: Matches the existing `q: i32` convention where call=+1, put=-1. Simple mapping.

### What NOT to Store in CSV

- **Implied volatility as the primary data**: Store raw prices. IV can be recomputed using the project's own `lets_be_rational` solver, which provides machine-precision results. Source-computed IVs may use different models or lower precision.
- **Log-moneyness, total variance**: These are derived quantities that depend on the forward price. Compute them in the parser, not in the CSV.
- **Greeks**: Derived from IV, not raw market data. Recompute via `black76` module.

## Git Tracking Decision

**Recommendation: Git-track the data/ directory.**

Rationale:

1. **Size**: A single day of SPX option chain data is roughly 3,000-5,000 rows (all strikes across all expiries). At ~100 bytes/row, that is 300-500 KB per file. The v1.2 milestone targets 1-3 days of data. Total: under 2 MB. This is well within git's comfort zone.

2. **Reproducibility**: The calibration library's correctness depends on being able to run against known data. If data files are gitignored, anyone cloning the repo cannot run validation without separately acquiring the data. For a curated, small dataset, git tracking ensures reproducibility.

3. **Stability**: These are historical snapshots -- they never change once committed. Git handles immutable blobs efficiently. There is no churn cost.

4. **No Git LFS needed**: Git LFS is for files exceeding ~50 MB or binary blobs that change frequently. CSVs under 1 MB each do not warrant the operational complexity of LFS.

**When to reconsider**: If the project later needs bulk historical data (hundreds of dates, tick-level data), add a `data-bulk/` directory to `.gitignore` and document download instructions in `data/README.md`. Keep the curated reference dataset in git-tracked `data/`.

### .gitignore Update

No changes needed for v1.2. The existing `.gitignore` only contains `/target`. The `data/` directory should be tracked.

If bulk data is added later:
```gitignore
/target
/data-bulk/
```

## Patterns to Follow

### Pattern 1: One File Per Observation Date

**What:** Each CSV file contains the complete option chain for a single underlying on a single trading date. The filename is the date in ISO 8601 format.

**When:** Always. This is the atomic unit.

**Why:** A single date's chain contains all the expiry slices needed for one SSVI surface calibration. Processing one file = one complete calibration run. No need to join across files.

**Example:**
```
data/cboe/spx/2024-01-19.csv   # All SPX options on Jan 19, 2024
data/cboe/spx/2024-06-21.csv   # All SPX options on Jun 21, 2024
```

### Pattern 2: Source-Faithful Column Names in Header

**What:** The CSV header uses the canonical column names defined above, but a comment or README documents the mapping from the source's original column names.

**When:** When downloading from CBOE, Eurex, or any provider whose raw format differs.

**Why:** Standardized headers mean the future parser only needs one CSV reading path per source (or ideally one universal path). Renaming columns during the manual download/preparation step is trivial and saves complexity in code.

**Example:**
```
# data/README.md excerpt:
# CBOE Optsum mapping:
#   quote_date     <- quote_date (same)
#   expiry         <- expiry (same)
#   strike         <- strike (same)
#   option_type    <- type (renamed)
#   bid            <- last_bid_price (renamed)
#   ask            <- last_ask_price (renamed)
#   underlying_price <- underlying_close (renamed)
#   forward        <- (not in source; compute from put-call parity)
#   discount_factor  <- (not in source; compute from rates)
```

### Pattern 3: Minimal Filtering at Storage Time

**What:** Store all strikes and expiries from the source, not just the subset useful for calibration. Filtering (removing illiquid strikes, excluding near-expiry options, trimming deep OTM wings) happens in the parser, not in the CSV.

**When:** Always.

**Why:** Filtering criteria evolve as the calibration improves. What is "too illiquid" depends on the weighting scheme. Storing the full chain preserves optionality. CSVs are small enough that the extra rows cost nothing.

### Pattern 4: data/README.md as Data Dictionary

**What:** A README.md in the data/ directory root documents: column definitions, units, sourcing instructions, date ranges available, known data quality issues, and licensing notes.

**When:** Created alongside the first data file.

**Why:** Six months from now, "where did this data come from and what do the columns mean" will not be obvious. The README is the single source of truth for data semantics.

**Example structure:**
```markdown
# Market Data

## Column Definitions
| Column | Type | Unit | Description |
|--------|------|------|-------------|
| quote_date | date | YYYY-MM-DD | Trading date |
| ...

## Sources
### CBOE DataShop
- URL: https://datashop.cboe.com/end-of-day-options-summary
- Product: Optsum (End-of-Day Options Summary)
- Cost: Free sample / paid subscription
- Column mapping: [table]

## Data Quality Notes
- SPX 2024-01-19: 3 contracts had bid > ask (removed)
- ...
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Storing Pre-Computed Implied Volatilities as Primary Data

**What:** Creating CSVs with columns like `iv` or `total_variance` instead of raw bid/ask prices.

**Why bad:** Ties the stored data to a specific IV computation method. The whole point of having `lets_be_rational` in the library is machine-precision IV recovery. If you store someone else's IV, you lose the ability to verify or improve the IV computation. Also, you cannot test the full pipeline (prices -> IV -> calibration) if the first step is pre-baked.

**Instead:** Store bid/ask prices. Compute mid-price and IV in the parser. Store source-provided IV as an optional column for cross-validation, not as the primary data.

### Anti-Pattern 2: Mixing Sources in One Directory

**What:** Putting CBOE data and Eurex data in the same directory with naming conventions to distinguish them.

**Why bad:** Different sources have different column formats, different underlying price conventions (spot vs settlement), different timestamp conventions. Mixing them creates ambiguity about which parsing rules apply.

**Instead:** Separate directories per source. Each source directory can have its own README section documenting format quirks.

### Anti-Pattern 3: Date Formats in Filenames That Are Not Sortable

**What:** Using `Jan-19-2024.csv` or `01-19-2024.csv` or `19012024.csv` as filenames.

**Why bad:** Not lexicographically sortable. Ambiguous (is `01-02-2024` January 2 or February 1?). Inconsistent with ISO 8601.

**Instead:** Always use `YYYY-MM-DD.csv`. Files sort chronologically when listed alphabetically.

### Anti-Pattern 4: Storing Derived Data Alongside Raw Data

**What:** Putting files like `spx_2024-01-19_surface.csv` (containing fitted parameters) in the same directory as raw option chain data.

**Why bad:** Conflates input data with output results. Makes it unclear what is source-of-truth market data vs. what was generated by the library.

**Instead:** Raw market data goes in `data/`. Calibration outputs go in `documents/` (as they already do). Clear separation of inputs and outputs.

## Integration With Existing Architecture

### Current Architecture Layers (Unchanged)

```
Model Layer (src/model/ssvi.rs)     -- pure SSVI formulas
Solver Layer (src/solver/)          -- Nelder-Mead, Brent
Calibration Layer (src/calibration.rs) -- orchestration
Binary Layer (src/bin/)             -- report generators
```

### New Layer: Data Layer (Future, v1.3+)

```
Data Layer (src/data/)              -- CSV parsing, record types
  |
  v
Calibration Layer (src/calibration.rs) -- receives CalibrationInput
```

The Data Layer sits **below** the Calibration Layer and **above** the filesystem. It reads CSV files and produces `CalibrationInput` structs. It does NOT depend on the Model or Solver layers. It DOES depend on:
- `src/pricing/lets_be_rational.rs` (to compute IV from prices)
- `src/pricing/black76.rs` (to compute forward from put-call parity)

### CalibrationInput Requirements

The existing `CalibrationInput` struct needs these fields per expiry slice:

```rust
pub struct CalibrationInput<'a> {
    pub k_slice: &'a [f64],     // log-moneyness: ln(K/F)
    pub w_market: &'a [f64],    // total variance: iv^2 * T
    pub theta_star: f64,        // ATM total variance
    pub k_star: f64,            // ATM log-moneyness reference
    pub weights: Option<&'a [f64]>,  // optional per-point weights
}
```

The future parser must produce these from raw CSV data. The pipeline:

1. **Read CSV** -> `Vec<OptionChainRecord>` (one record per row)
2. **Determine forward** per expiry (from CSV column, or via put-call parity)
3. **Compute mid-price** = (bid + ask) / 2
4. **Compute IV** per option using `implied_vol(price, forward, strike, t, q)`
5. **Compute log-moneyness** `k = ln(K/F)` per option
6. **Compute total variance** `w = iv^2 * T` per option
7. **Group by expiry** -> one `CalibrationInput` per expiry slice
8. **Identify ATM** -> `theta_star` and `k_star` from the option closest to K=F

This pipeline requires no changes to the existing calibration code. `CalibrationInput` already accepts the right shape of data.

### Binary Integration (Future)

A new binary `src/bin/fit_market.rs` would replace the synthetic data generation in `fit_real.rs`:

```rust
// FUTURE: Replace build_market_slices() with:
// let slices = essvi::data::load_chain("data/cboe/spx/2024-01-19.csv")?;
// let inputs = essvi::data::to_calibration_inputs(&slices)?;
```

The existing `FitResult` struct and `plot_fit()` function work unchanged.

## Scalability Considerations

| Concern | v1.2 (1-3 files) | v1.3 (10-20 files) | Future (100+ files) |
|---------|-------------------|---------------------|---------------------|
| Storage | <2 MB, git-tracked | <10 MB, git-tracked | Git LFS or gitignored `data-bulk/` |
| File discovery | Hardcoded path in binary | Glob `data/{source}/{underlying}/*.csv` | Index file or CLI argument |
| Parse time | <100ms | <1s | Parallel parsing with rayon |
| Memory | All in memory | All in memory | Stream or lazy-load |

For v1.2, none of the scaling concerns apply. The architecture supports growth without restructuring.

## Sources

- [CBOE End-of-Day Options Summary](https://datashop.cboe.com/end-of-day-options-summary) -- CBOE DataShop product page documenting Optsum CSV columns (quote_date, underlying_symbol, root, expiry, strike, type, open_interest, total_volume, last_bid_price, last_ask_price, etc.)
- [Beyond Surrogate Modeling -- data/ directory](https://github.com/mChataign/Beyond-Surrogate-Modeling-Learning-the-Local-Volatility-Via-Shape-Constraints) -- Reference SVI calibration project with data/ directory containing DAX, Euro Stoxx 50, and SPX option chain data files organized by date
- [py_ssvi -- SSVI calibration with iv.csv](https://github.com/rfbressan/py_ssvi) -- Reference SSVI calibration project storing implied volatility data in CSV with columns: period, moneyness, iv
- [SVI and SSVI Volatility Surface Fitting](https://github.com/chi-gamma/SVI_and_SSVI_Volatility_Surface_fitting) -- Reference project for SVI/SSVI/eSSVI parametrization and calibration to raw option data
- [Eurex Euro Stoxx 50 Index Options](https://www.eurex.com/ex-en/markets/idx/stx/euro-stoxx-50-derivatives/products/EURO-STOXX-50-Index-Options-46548) -- Eurex product page for Euro Stoxx 50 options (European exercise, settlement conventions)
- [Git LFS documentation](https://git-lfs.com/) -- Git Large File Storage guidance; confirms LFS unnecessary for files under ~50 MB that do not change

---

*Architecture analysis: 2026-03-07*
