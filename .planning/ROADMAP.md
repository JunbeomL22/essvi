# Roadmap: essvi

## Milestones

- v1.0 Idiomatic Restructuring - Phases 1-5 (shipped 2026-03-07)
- v1.1 Pricing Primitives - Phases 6-8 (in progress)

## Phases

<details>
<summary>v1.0 Idiomatic Restructuring (Phases 1-5) - SHIPPED 2026-03-07</summary>

- [x] Phase 1: Module Restructuring (1/1 plans) - completed 2026-03-07
- [x] Phase 2: Error Types and Impl Blocks (1/1 plans) - completed 2026-03-07
- [x] Phase 3: Calibration Config (1/1 plans) - completed 2026-03-07
- [x] Phase 4: Binary Deduplication (1/1 plans) - completed 2026-03-07
- [x] Phase 5: Test Migration (1/1 plans) - completed 2026-03-07

</details>

### v1.1 Pricing Primitives

- [x] **Phase 6: Math Foundations** - Error functions, normal distributions, and numerical constants
- [ ] **Phase 7: Black-76 Pricing** - Undiscounted/discounted pricing, greeks, and PricingError type
- [ ] **Phase 8: Implied Volatility Solver** - Let's Be Rational algorithm with rational cubic initial guesses

## Phase Details

### Phase 6: Math Foundations
**Goal**: Library provides the mathematical building blocks that Black-76 and implied volatility depend on
**Depends on**: Nothing (first phase of v1.1; builds on v1.0 module structure)
**Requirements**: MATH-01, MATH-02, MATH-03, MATH-05
**Success Criteria** (what must be TRUE):
  1. Calling erf, erfc, and erfcx returns values matching reference tables to machine precision
  2. Standard normal PDF, CDF, and inverse CDF produce correct values across the full domain including tails
  3. High-precision normal CDF with asymptotic expansion handles extreme arguments without returning 0 or 1 prematurely
  4. Numerical constants module exists and is importable by downstream modules
**Plans**: 1/1

Plans:
- [x] 06-01: Implement math foundations (erf, normal, normal_hp, constants) - completed 2026-03-07

### Phase 7: Black-76 Pricing
**Goal**: Users can price futures options and compute greeks using the Black-76 model
**Depends on**: Phase 6
**Requirements**: BLK-01, BLK-02, BLK-03, BLK-04, BLK-05
**Success Criteria** (what must be TRUE):
  1. User can compute undiscounted call and put prices and deltas given forward, strike, sigma, and time
  2. User can compute gamma, vega, and theta individually, and retrieve all greeks in a single combined call
  3. User can compute a discounted option price by passing a discount factor
  4. Invalid inputs and out-of-bounds prices return typed PricingError variants, not panics
**Plans**: TBD

Plans:
- [ ] 07-01: TBD
- [ ] 07-02: TBD

### Phase 8: Implied Volatility Solver
**Goal**: Users can recover implied volatility from option prices at machine precision
**Depends on**: Phase 6, Phase 7 (uses PricingError)
**Requirements**: MATH-04, IVOL-01, IVOL-02, IVOL-03, IVOL-04
**Success Criteria** (what must be TRUE):
  1. User can compute Black implied volatility from a market option price, forward, strike, and time to expiry
  2. Normalised implied volatility from normalised price is available as a lower-level entry point
  3. Rational cubic interpolation produces accurate initial guesses that the Householder iterations refine
  4. Algorithm converges to machine-precision implied volatility in at most 2 Householder iterations
  5. Edge cases (deep ITM, deep OTM, at-the-money) return correct results without numerical blowup
**Plans**: TBD

Plans:
- [ ] 08-01: TBD
- [ ] 08-02: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 6 -> 7 -> 8

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Module Restructuring | v1.0 | 1/1 | Complete | 2026-03-07 |
| 2. Error Types and Impl Blocks | v1.0 | 1/1 | Complete | 2026-03-07 |
| 3. Calibration Config | v1.0 | 1/1 | Complete | 2026-03-07 |
| 4. Binary Deduplication | v1.0 | 1/1 | Complete | 2026-03-07 |
| 5. Test Migration | v1.0 | 1/1 | Complete | 2026-03-07 |
| 6. Math Foundations | v1.1 | 1/1 | Complete | 2026-03-07 |
| 7. Black-76 Pricing | v1.1 | 0/? | Not started | - |
| 8. Implied Volatility Solver | v1.1 | 0/? | Not started | - |
