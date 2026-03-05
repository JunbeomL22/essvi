# essvi — Extended SVI Calibration Library

## What This Is

A pure Rust library for calibrating implied volatility surfaces using SVI-family parameterizations (SSVI and eSSVI). Provides derivative-free optimization (bounded Nelder-Mead), implicit theta resolution (Brent root-finding), and no-arbitrage enforcement. Targets quantitative finance practitioners who need fast, correct vol surface fitting with zero external dependencies.

## Core Value

Accurate, arbitrage-free implied volatility surface calibration that handles real-market skew profiles including steep put-side skew at all expiries.

## Requirements

### Validated

- SSVI model (phi function, total variance formula, no-arb check)
- Bounded Nelder-Mead optimizer (2D/3D with projection)
- Brent root finder for implicit theta solving
- SSVI calibration pipeline (rho-grid sweep + 2D NM + 3D polish)
- Benchmark report generator with SVG plots
- Inline unit tests and integration stress tests

### Active

- [ ] eSSVI model implementation (coexists with SSVI)
- [ ] eSSVI calibration pipeline
- [ ] Comparative fit quality report (SSVI vs eSSVI)

### Out of Scope

- Replacing existing SSVI implementation — coexistence, not replacement
- Surface-level calibration (cross-slice consistency) — future milestone
- Real market data parsing — future milestone
- API ergonomics / crate publishing — future milestone

## Context

- The current SSVI implementation has a known limitation: at long expiry (T=1) with steep skew (slope >= 0.8), the no-arb constraint eta*(1+|rho|) <= 2 forces eta to saturate at 2.0 and rho to collapse to 0, producing 900-1700 bps IV errors.
- eSSVI (Hendriks & Martini) reparameterizes the surface with different/relaxed constraints that should handle wings better.
- The benchmark target smile is synthetic: sigma(k) = atm_vol + slope*(-k)^+ + 0.1*slope*k^+ + curvature*k^2.

## Constraints

- **Tech stack**: Pure Rust, zero external dependencies for core logic (plotters allowed for reporting only)
- **Architecture**: eSSVI must coexist as separate module, not modify existing SSVI code
- **Testing**: All new code must have inline unit tests

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Bounded Nelder-Mead over L-BFGS-B | Only 3 vars, avoids external C deps | Good |
| Eliminate equality constraint via implicit theta | Simpler solver, better convergence | Good |
| Coexist eSSVI alongside SSVI | Non-breaking, users choose parameterization | -- Pending |

## Current Milestone: v1.0 eSSVI Implementation

**Goal:** Add extended SSVI parameterization with better wing/skew handling, demonstrated via comparative fit quality report.

**Target features:**
- eSSVI model module
- eSSVI calibration pipeline
- Comparative fit quality report (SSVI vs eSSVI)

---
*Last updated: 2026-03-05 after milestone v1.0 initialization*
