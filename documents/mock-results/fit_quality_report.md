# SSVI Slice Fit Quality Report

ATM vol = 0.4  |  k range = [-0.4, 0.4]  |  k\* = 0.01

## 1. Expiry (T) vs Skew Slope

| T | Slope | max IV err (bps) | RMSE IV (bps) | eta | gamma | rho | phi | eta*(1+\|rho\|) | converged |
|---:|------:|----------------:|--------------:|----:|------:|----:|----:|---------------:|:---------:|
| 0.03 | 0.5 | 215.1 | 101.9 | 0.537 | 0.491 | -0.146 | 7.32 | 0.616 | yes |
| 0.03 | 1 | 296.2 | 142.6 | 0.534 | 0.606 | -0.216 | 13.38 | 0.650 | yes |
| 0.1 | 0.5 | 215.1 | 101.9 | 0.548 | 0.630 | -0.146 | 7.32 | 0.628 | yes |
| 0.1 | 1 | 296.2 | 142.6 | 0.537 | 0.783 | -0.216 | 13.38 | 0.654 | yes |
| 0.25 | 0.5 | 215.1 | 101.9 | 0.608 | 0.778 | -0.146 | 7.32 | 0.697 | yes |
| 0.25 | 1 | 296.2 | 142.6 | 0.601 | 0.972 | -0.216 | 13.38 | 0.731 | yes |
| 0.5 | 0.5 | 215.1 | 101.9 | 0.595 | 0.997 | -0.146 | 7.32 | 0.682 | yes |
| 0.5 | 1 | 296.2 | 142.6 | 1.097 | 1.000 | -0.216 | 13.38 | 1.334 | yes |
| 1 | 0.2 | 125.1 | 61.7 | 0.610 | 0.997 | -0.088 | 3.78 | 0.664 | yes |
| 1 | 0.4 | 195.9 | 92.3 | 0.992 | 1.000 | -0.129 | 6.16 | 1.120 | yes |

## 2. Fit Plots (all combinations)

**T=0.03, slope=0.5** — max err: 215 bps, eta=0.537, rho=-0.146, eta*(1+|rho|)=0.616

![T0p03_s0p5](plots/fit_T0p03_s0p5.svg)

**T=0.03, slope=1** — max err: 296 bps, eta=0.534, rho=-0.216, eta*(1+|rho|)=0.650

![T0p03_s1](plots/fit_T0p03_s1.svg)

**T=0.1, slope=0.5** — max err: 215 bps, eta=0.548, rho=-0.146, eta*(1+|rho|)=0.628

![T0p1_s0p5](plots/fit_T0p1_s0p5.svg)

**T=0.1, slope=1** — max err: 296 bps, eta=0.537, rho=-0.216, eta*(1+|rho|)=0.654

![T0p1_s1](plots/fit_T0p1_s1.svg)

**T=0.25, slope=0.5** — max err: 215 bps, eta=0.608, rho=-0.146, eta*(1+|rho|)=0.697

![T0p25_s0p5](plots/fit_T0p25_s0p5.svg)

**T=0.25, slope=1** — max err: 296 bps, eta=0.601, rho=-0.216, eta*(1+|rho|)=0.731

![T0p25_s1](plots/fit_T0p25_s1.svg)

**T=0.5, slope=0.5** — max err: 215 bps, eta=0.595, rho=-0.146, eta*(1+|rho|)=0.682

![T0p5_s0p5](plots/fit_T0p5_s0p5.svg)

**T=0.5, slope=1** — max err: 296 bps, eta=1.097, rho=-0.216, eta*(1+|rho|)=1.334

![T0p5_s1](plots/fit_T0p5_s1.svg)

**T=1, slope=0.2** — max err: 125 bps, eta=0.610, rho=-0.088, eta*(1+|rho|)=0.664

![T1_s0p2](plots/fit_T1_s0p2.svg)

**T=1, slope=0.4** — max err: 196 bps, eta=0.992, rho=-0.129, eta*(1+|rho|)=1.120

![T1_s0p4](plots/fit_T1_s0p4.svg)

## 3. No-Arbitrage Constraint Saturation

The SSVI no-arb condition `eta * (1 + |rho|) <= 2` limits how much skew the model can produce.
When this value approaches 2.0, the optimizer is constrained and fit quality degrades.

| T | Slope | eta | rho | eta*(1+\|rho\|) | saturated | max err (bps) |
|---:|------:|----:|----:|---------------:|:---------:|--------------:|
| 0.03 | 0.5 | 0.537 | -0.146 | 0.616 | no | 215 |
| 0.03 | 1 | 0.534 | -0.216 | 0.650 | no | 296 |
| 0.1 | 0.5 | 0.548 | -0.146 | 0.628 | no | 215 |
| 0.1 | 1 | 0.537 | -0.216 | 0.654 | no | 296 |
| 0.25 | 0.5 | 0.608 | -0.146 | 0.697 | no | 215 |
| 0.25 | 1 | 0.601 | -0.216 | 0.731 | no | 296 |
| 0.5 | 0.5 | 0.595 | -0.146 | 0.682 | no | 215 |
| 0.5 | 1 | 1.097 | -0.216 | 1.334 | no | 296 |
| 1 | 0.2 | 0.610 | -0.088 | 0.664 | no | 125 |
| 1 | 0.4 | 0.992 | -0.129 | 1.120 | no | 196 |
