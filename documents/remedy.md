# Calendar Arbitrage Remedy for Per-Slice SSVI Calibration

## Problem

Per-slice (η, γ, ρ) calibration requires preventing calendar spread arbitrage.
The formal Hendriks-Martini conditions require (ρ, ψ) reparameterization,
but since φ(θ) = η / (θ^γ (1+θ)^(1-γ)) keeps ψ as a derived quantity (not a free variable), direct application is impractical.

## Remedy

Add a calendar arbitrage penalty term to the objective function.

### Loss Function

$$
L = \underbrace{\| W - w \|^2}_{\text{fit error}} + \lambda \underbrace{\sum_{j} \max\!\left(0,\; w_{\text{prev}}(k_j) - w(k_j)\right)^2}_{\text{calendar arbitrage penalty}}
$$

- $w_{\text{prev}}(k_j)$: total variance of the previous (already calibrated) maturity slice
- $w(k_j)$: total variance of the current slice being calibrated
- $k_j$: sample points covering the strike range (10-20 points, including wings)
- $\lambda$: penalty weight (sufficiently large, e.g. 100-1000)

When there is no violation, the penalty is zero and fit quality is unaffected.
When violated, the optimizer naturally steers away from the crossing region.
