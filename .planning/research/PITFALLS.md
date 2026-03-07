# Domain Pitfalls: Real Option Price Data for Vol Surface Calibration

**Domain:** Collecting and storing real option price data for SVI/SSVI calibration
**Researched:** 2026-03-07
**Confidence:** HIGH (well-documented domain with extensive academic and practitioner literature)

## Critical Pitfalls

Mistakes that cause wrong calibration results, silently corrupt the vol surface, or require rearchitecting data pipelines.

### Pitfall 1: Using American-Style Option Prices with Black-76

**What goes wrong:** The essvi library uses Black-76 (European-style forward pricing) and Let's Be Rational (European IV solver). Feeding American-style option prices into these models produces systematically biased implied volatilities. American puts carry early exercise premium that inflates their price above the European equivalent, producing IVs that are too high. For ITM American puts near ex-dividend dates, the bias can be 1-3 vol points or more.

**Why it happens:** The most liquid US equity options (SPY, individual stocks) are American-style. SPX is European-style but is sometimes confused with SPY. Data sources rarely flag exercise style prominently.

**Consequences:** The IV solver will return values that include the early exercise premium baked in as spurious volatility. The SVI calibration will fit a skew that is artificially steeper on the put side. The resulting vol surface systematically overprices puts and underprices calls when used for European pricing.

**Prevention:**
- Use only European-style index options: SPX (CBOE), Euro Stoxx 50 (Eurex), Nikkei 225 (JPX/OSE), KOSPI 200 (KRX).
- Verify exercise style in contract specifications before collecting data. SPX is European; SPY is American. This distinction matters.
- If American data is the only option available, use only OTM options where early exercise premium is negligible (but this is a compromise, not a solution).
- Store exercise style as a metadata field in the data file so downstream consumers can validate.

**Detection:** Compare put-call parity residuals. European options should satisfy `C - P = DF * (F - K)` precisely. Large residuals on the put side (puts too expensive relative to calls) signal early exercise premium contamination.

**Confidence:** HIGH. This is a textbook distinction. SPX is confirmed European-style by CBOE. Euro Stoxx 50 is confirmed European-style by Eurex. Nikkei 225 is confirmed European-style by JPX.

### Pitfall 2: Using Mid-Price from Illiquid Strikes

**What goes wrong:** Deep OTM and deep ITM options have wide bid-ask spreads and low volume. The "mid-price" (average of bid and ask) for these strikes is not a reliable estimate of the fair option price. It can be systematically biased or simply noisy, and using it for IV calculation injects large errors into the wings of the smile.

**Why it happens:** Deep OTM options have very small absolute prices (sometimes $0.05-$0.50 for index options), so a bid-ask spread of $0.10-$0.50 represents 20-100% of the option value. Market makers widen spreads for strikes with low volume and open interest. End-of-day snapshots may show stale quotes where one side was posted hours ago.

**Consequences:** IV computed from illiquid mid-prices produces noisy, unreliable wing points. The SVI calibration optimizer tries to fit through these noisy points, either distorting the entire smile or producing unstable parameters. Specifically, for SSVI calibration:
- Noisy wing data destabilizes eta and rho estimates.
- The Nelder-Mead optimizer may chase outliers in the wings.
- Calendar arbitrage violations can appear because different expiries have different noise profiles.

**Prevention:**
- **Filter by bid-ask spread:** Exclude options where `(ask - bid) / mid > threshold`. A threshold of 0.50 (50% relative spread) is aggressive; 0.33 (33%) is common in academic literature. For this project, start with 0.50 and tighten if data quality allows.
- **Filter by open interest:** Require minimum open interest (at least 10-100 contracts depending on the underlying). Options with zero open interest should always be excluded.
- **Filter by volume:** Prefer strikes with nonzero daily volume. Zero-volume strikes may have stale quotes.
- **Filter by zero bid:** Exclude any option with a bid of zero. A zero bid means no one is willing to buy at any price -- the mid-price is meaningless.
- **Use OTM options only:** For a given strike, use OTM calls above ATM and OTM puts below ATM. OTM options have more time value relative to their price, making IV extraction more numerically stable. This is standard practice in vol surface construction.
- **Weight by vega in calibration:** If noisy wing points make it through filters, the calibration should weight them less. The existing `weights` parameter in `CalibrationInput` supports this -- assign lower weights to far-OTM strikes.

**Detection:** Plot IV vs. log-moneyness before calibration. Outliers in the wings (points that deviate > 5-10 vol points from neighbors) indicate illiquid or stale data. Automated detection: flag any point where `|IV_i - IV_{i-1}| > 0.10` (10 vol points) between adjacent strikes.

**Confidence:** HIGH. Extensively documented in academic literature (Wallmeier 2024, Springer 2023). Standard practice in industry.

### Pitfall 3: Wrong Forward Price

**What goes wrong:** The SSVI model parameterizes the smile in log-forward-moneyness `k = ln(K/F)`, where F is the forward price. Using the wrong forward price shifts the entire smile horizontally, misidentifying the ATM point and corrupting `theta_star` (ATM total variance) and `k_star` (ATM log-moneyness). This silently produces a miscalibrated surface.

**Why it happens:** The forward price is not directly observable from option chain data. It must either be:
1. Extracted from put-call parity: `F = K + e^{rT} * (C - P)` at the strike where call and put prices are closest.
2. Read from the corresponding futures contract (if one exists and is liquid).
3. Computed as `F = S * e^{(r-q)T}` using spot, risk-free rate, and dividend yield -- but this requires knowing the correct rate and dividend yield.

Common errors:
- Using spot price instead of forward price.
- Using an approximate or wrong risk-free rate.
- Ignoring or incorrectly estimating the dividend yield for equity indices.
- Using a futures price from a different expiry than the options expiry.

**Consequences:**
- The log-moneyness values `k = ln(K/F)` are all shifted by a constant bias.
- `theta_star` and `k_star` fed to the calibrator are wrong.
- The solve_theta function may fail (NonPositiveTheta, ThetaDivergence) or converge to nonsensical parameters.
- Systematic pricing errors in both directions when the calibrated surface is used.

**Prevention:**
- **Preferred: Extract forward from put-call parity.** For each expiry, find the strike K* where `|C(K*) - P(K*)|` is minimized. Then `F = K* + e^{rT} * (C(K*) - P(K*))`. This only requires the risk-free rate (not dividend yield) and is self-consistent with the option prices.
- **Alternative: Use the corresponding futures settlement price** if available (e.g., Euro Stoxx 50 futures settle on Eurex alongside the options).
- **Store the forward price** in the data file alongside strikes and prices. Do not leave it to be computed at calibration time without documentation of the method used.
- **Cross-check:** The implied forward should satisfy `F ~ S * e^{(r-q)T}` approximately. Large deviations (> 1%) suggest an error in one of the inputs.

**Detection:** Compute put-call parity residuals `|C - P - DF*(F - K)|` across all strikes. These should be near zero (< 1-2 ticks) for European options. If they show a systematic bias, the forward is wrong.

**Confidence:** HIGH. Forward extraction from put-call parity is the standard approach in the Gatheral SVI framework and is explicitly assumed in the SSVI parameterization.

### Pitfall 4: Non-Synchronous Data (Staleness)

**What goes wrong:** Option prices at different strikes are observed at different times. End-of-day data snapshots may show the bid/ask at 3:59 PM for some strikes and at 2:30 PM for others (if the latter did not trade near the close). Meanwhile, the underlying index or futures price used as "current" reflects the 4:00 PM close. This time mismatch introduces artificial noise and can violate no-arbitrage conditions.

**Why it happens:** Illiquid OTM options may not trade for hours. Quote updates may be sparse. The OptionMetrics IvyDB database, a major academic data source, records prices at 3:59 PM, not 4:00 PM (Wallmeier 2024). If the underlying moves between 3:59 and 4:00, all IVs are biased.

**Consequences:**
- Artificial violations of put-call parity.
- Implied forward price extraction becomes noisy.
- The IV smile appears jagged or non-monotone when it should be smooth.
- SVI calibration may struggle to fit through data points that reflect different underlying levels.

**Prevention:**
- Use settlement prices from the exchange rather than last trade prices. Settlement prices are computed by the exchange to reflect consistent end-of-day levels, accounting for bid/ask and order flow.
- For manually collected data, prefer "closing bid/ask" over "last trade" for each strike.
- Record the timestamp or source of each price in the data file (settlement vs. last trade vs. bid/ask snapshot).
- If using last trade prices, record the trade timestamp and exclude strikes where the last trade was more than 30 minutes before the close.

**Detection:** Look for put-call parity violations that are inconsistent across strikes. If the implied forward varies widely depending on which strike pair is used, the data is likely non-synchronous.

**Confidence:** HIGH. The Wallmeier (2024) study in the Journal of Futures Markets specifically documents this issue for OptionMetrics data.

## Moderate Pitfalls

### Pitfall 5: Wrong Time-to-Expiry Calculation

**What goes wrong:** Time-to-expiry `T` in years is used in total variance `w = sigma^2 * T` and is a direct input to Black-76. Off-by-one errors, wrong day count conventions, or confusing calendar days with trading days produce systematic bias in the vol surface.

**Prevention:**
- Use calendar days / 365 (ACT/365) as the day count convention, which is standard for equity index options in most markets. This is what Black-76 assumes.
- Count from the observation date to the expiry date (inclusive of the observation date, exclusive of the expiry date, or vice versa -- be consistent and document the convention).
- For very short expiries (< 7 days), even a 1-day error in T changes the IV by 5-10% or more. Consider excluding expiries shorter than 7 calendar days unless the data quality is excellent.
- Store the raw dates (observation date, expiry date) in the data file so T can be recomputed if needed.

**Confidence:** HIGH. Standard quantitative finance issue.

### Pitfall 6: Ignoring or Mishandling the Discount Factor

**What goes wrong:** Black-76 pricing uses undiscounted prices `C_undiscounted = DF^{-1} * C_market` where `DF = e^{-rT}`. The essvi IV solver (`implied_volatility`) expects undiscounted prices. If market prices (which are discounted) are passed directly without dividing by DF, the resulting IVs are wrong.

**Prevention:**
- Decide early: will the data file store discounted (market) prices or undiscounted prices?
- If discounted (most natural for raw market data): the parsing/calibration pipeline must divide by `DF = e^{-rT}` before calling the IV solver.
- Store the risk-free rate alongside the data so the discount factor can be computed.
- For short-dated options (T < 0.25 years), the discount factor is very close to 1.0 and the error is small. For T > 1 year, ignoring discounting introduces meaningful IV error.
- Use the same rate source consistently (e.g., SOFR for USD, ESTR for EUR, TONAR for JPY).

**Confidence:** HIGH. Directly follows from the Black-76 model specification in the essvi codebase (`src/pricing/black76.rs` lines 231-266).

### Pitfall 7: Mixing Moneyness Conventions (Spot vs. Forward, Strike vs. Log-Moneyness)

**What goes wrong:** The SSVI model uses log-forward-moneyness `k = ln(K/F)`. Data sources may provide: raw strike prices K, spot-moneyness `K/S`, forward-moneyness `K/F`, log-spot-moneyness `ln(K/S)`, or log-forward-moneyness `ln(K/F)`. Using the wrong convention silently shifts the smile.

**Prevention:**
- Store raw strikes K in the data file. Convert to log-forward-moneyness only at calibration time, using the forward price also stored in the data.
- Never store pre-computed log-moneyness without also storing the forward price used to compute it.
- Document the moneyness convention in the data file header or metadata.

**Confidence:** HIGH. The SSVI model definition in `src/model/ssvi.rs` uses `k` as log-forward-moneyness.

### Pitfall 8: Insufficient Strike Coverage per Expiry

**What goes wrong:** SVI/SSVI calibration needs sufficient data points across the smile to determine all parameters. With fewer than 5-6 strikes per expiry, the 3-parameter SSVI optimization (eta, gamma, rho) is underdetermined or poorly conditioned. The existing code uses 20-point synthetic slices; real data may have far fewer liquid strikes.

**Prevention:**
- Require a minimum of 8-10 strikes per expiry after filtering. If an expiry has fewer than 5 usable strikes, exclude it from calibration.
- Strikes should span both sides of ATM. Having all strikes on one side makes rho estimation unreliable.
- Record the total number of listed strikes vs. the number passing quality filters in the data metadata.

**Detection:** After filtering, count strikes per expiry. Flag expiries with fewer than 8 strikes or where all strikes are on one side of ATM.

**Confidence:** HIGH. The existing calibration code (`fit_real.rs`) uses 40 points per slice; real liquid index options typically have 20-50 liquid strikes per expiry.

### Pitfall 9: Treating Settlement Prices as Last Trade Prices (or Vice Versa)

**What goes wrong:** Settlement prices and last trade prices serve different purposes and can differ significantly, especially for illiquid contracts. Settlement prices are computed by the exchange using a methodology (VWAP, committee-determined, or model-based) designed to reflect fair value at the close. Last trade prices reflect the most recent transaction, which may have occurred hours before the close.

**Prevention:**
- Prefer settlement prices over last trade prices for end-of-day data. Settlement prices are more representative and internally consistent across strikes.
- When manually downloading data, verify which price type is being shown. Yahoo Finance typically shows last trade prices. Exchange websites (Eurex, CBOE, JPX) provide official settlement prices.
- Record the price type (settlement, last, bid, ask, mid) in the data file metadata.

**Confidence:** HIGH. The CME Group documentation explicitly describes settlement price calculation methodology as distinct from last trade prices.

## Minor Pitfalls

### Pitfall 10: Ignoring Index Dividend Yield

**What goes wrong:** For equity index options, the forward price `F = S * e^{(r-q)T}` depends on the continuous dividend yield `q`. For Euro Stoxx 50 and Nikkei 225, dividend yields of 2-4% shift the forward price materially for T > 0.25 years. Ignoring dividends (setting q=0) when computing the forward from spot introduces a systematic downward bias in F.

**Prevention:**
- Extract the forward from put-call parity (method 1 in Pitfall 3) rather than computing from spot. This avoids needing to know the dividend yield.
- If computing F from spot, use the actual expected dividend yield for the observation date, not an annual average. Equity index dividend yields are seasonal (higher yield from April-June for European indices).

**Confidence:** MEDIUM. Impact depends on the method used for forward extraction. If using put-call parity, this pitfall is avoided entirely.

### Pitfall 11: Data File Format Ambiguities

**What goes wrong:** CSV files without clear headers, units, or metadata lead to misinterpretation. Is the strike in index points or dollars? Is IV in decimal (0.20) or percentage (20.0)? Is the price per contract or per index point?

**Prevention:**
- Use clear column headers: `strike`, `bid`, `ask`, `mid`, `type` (C/P), `expiry`, `forward`, `rate`.
- Document units in a header comment or companion metadata: prices in index points, IV in decimal, T in years, rate as continuous compound rate.
- Include a data dictionary or schema comment at the top of each data file.
- Store one expiry per file or clearly delimit expiries within a file.

**Confidence:** HIGH. Standard data engineering practice.

### Pitfall 12: Including Near-Expiry Options (T < 7 Days)

**What goes wrong:** Very short-dated options have almost no time value, making IV extraction numerically unstable. Small absolute price errors (even 1 tick) translate to large IV errors. The SSVI model is designed for smooth smiles, but near-expiry smiles can be extremely steep or erratic.

**Prevention:**
- Exclude options with fewer than 7 calendar days to expiry from the initial data collection.
- If near-expiry data is included for research purposes, flag it as unreliable and do not use it for SSVI calibration without specialized handling.

**Confidence:** HIGH. The Let's Be Rational solver in the codebase handles `t -> 0` gracefully, but the IV values themselves become meaningless for calibration at very short tenors.

### Pitfall 13: Prices Below Intrinsic Value

**What goes wrong:** Some market data snapshots show option prices that appear to violate the no-arbitrage condition `C >= max(F - K, 0)` (for undiscounted prices). These prices are artifacts of non-synchronous data, wide spreads, or stale quotes. The Let's Be Rational solver will return `Err(PricingError::BelowIntrinsic)` for these, which is correct -- but if not handled, they create gaps in the smile.

**Prevention:**
- Pre-filter: exclude any option where `mid < intrinsic - epsilon` (where epsilon allows for 1-2 ticks of noise).
- The essvi solver already has `BelowIntrinsic` error handling (`src/pricing/error.rs`). The data pipeline should expect and handle these gracefully rather than treating them as bugs.
- Log excluded data points for debugging.

**Confidence:** HIGH. Directly addressed in the existing codebase.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Data collection (v1.2) | Pitfall 1 (American vs European) | Verify exercise style before downloading. SPX, Euro Stoxx 50, Nikkei 225 are all European. |
| Data collection (v1.2) | Pitfall 3 (Wrong forward) | Store enough auxiliary data (spot, rate, or futures price) to compute the forward. Better: store the forward derived from put-call parity alongside the option data. |
| Data collection (v1.2) | Pitfall 11 (Format ambiguity) | Define a clear data schema before downloading. Include units and metadata. |
| Data collection (v1.2) | Pitfall 9 (Settlement vs last trade) | Record which price type is being stored. Prefer settlement prices. |
| Data parsing (future) | Pitfall 2 (Illiquid strikes) | Implement bid-ask spread, open interest, and volume filters. |
| Data parsing (future) | Pitfall 7 (Moneyness convention) | Convert raw strikes to log-forward-moneyness at parsing time using the stored forward price. |
| IV extraction (future) | Pitfall 6 (Discount factor) | Divide market prices by DF before calling the IV solver. |
| IV extraction (future) | Pitfall 13 (Below intrinsic) | Handle `BelowIntrinsic` errors by excluding those strikes from calibration. |
| Calibration integration (future) | Pitfall 8 (Strike coverage) | Check per-expiry strike count after filtering. Require 8+ strikes. |
| Calibration integration (future) | Pitfall 5 (Time to expiry) | Use ACT/365. Exclude T < 7 days. |

## Summary of Filtering Criteria (Reference for Implementation)

When data parsing and IV extraction are implemented (future milestones), apply these filters in order:

1. **Exercise style:** European only. Reject American-style contracts.
2. **Zero bid:** Exclude any option with bid = 0.
3. **Relative spread:** Exclude if `(ask - bid) / mid > 0.50`.
4. **Open interest:** Exclude if open interest < 10.
5. **OTM only:** Use OTM calls (K > F) and OTM puts (K < F). Discard ITM options.
6. **Intrinsic value:** Exclude if mid < intrinsic value (adjusted for discounting).
7. **Neighbor consistency:** Flag if `|IV_i - IV_{i-1}| > 0.10` (10 vol points) between adjacent strikes.
8. **Minimum strikes per expiry:** Require >= 8 strikes per expiry after all filters. Discard expiries with fewer.
9. **Minimum time to expiry:** Exclude T < 7 calendar days.

## Sources

- [Wallmeier (2024): Quality Issues of Implied Volatilities in OptionMetrics IvyDB](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4025257) - Data quality issues in major academic options database (timing discrepancies, put-call parity violations, dividend errors)
- [Nagy & Ormos (2017): Volatility Surface Calibration in Illiquid Market Environment](https://www.scs-europe.net/dlib/2017/ecms2017acceptedpapers/0148-fes_ECMS2017_0109.pdf) - Illiquidity effects on SVI calibration, vega weighting
- [Gatheral & Jacquier (2014): Arbitrage-free SVI volatility surfaces](https://arxiv.org/pdf/1204.0646) - SSVI parameterization, log-forward-moneyness convention
- [Corbetta et al. (2023): Unbiasing and robustifying implied volatility calibration](https://hal.science/hal-03715921v1/file/main.pdf) - Bid-ask spread effects on calibration, mid-price bias
- [CBOE: SPX Settlement](https://cdn.cboe.com/resources/spx/Settlement_of_Standard_AM_Settled_SP_500_Index_Options.pdf) - A.M. settlement and SOQ methodology
- [Eurex: Euro Stoxx 50 Index Options](https://www.eurex.com/ex-en/markets/idx/stx/euro-stoxx-50-derivatives/products/EURO-STOXX-50-Index-Options-46548) - European-style, Eurex settlement
- [JPX: Nikkei 225 Options Contract Specifications](https://www.jpx.co.jp/english/derivatives/products/domestic/225options/01.html) - European-style, JPX settlement
- [CME Group: About Settlements](https://www.cmegroup.com/trading/about-settlements.html) - Settlement price vs. last trade price methodology
- [SPX vs SPY Options Comparison](https://marketxls.com/blog/spx-vs-spy-options-a-comprehensive-comparison) - European (SPX) vs. American (SPY) exercise style differences
- [Springer (2023): Implied volatility surfaces using half a billion option prices](https://link.springer.com/article/10.1007/s11147-023-09195-5) - Large-scale IV surface construction, filtering methodology
