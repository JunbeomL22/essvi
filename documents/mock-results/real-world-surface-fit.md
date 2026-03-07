# SSVI Real-World Surface Fit Report

Surface calibration with calendar arbitrage penalty.

- **Step 1**: Unconstrained per-slice fit (3x weight for k in [-0.2, 0.2])
- **Step 2**: Sequential refit with calendar penalty (lambda=100, k penalty points: -0.5 to 0.5, step 0.05)

## Step 1: Unconstrained Fit

| T | max IV err (bps) | RMSE IV (bps) | eta | gamma | rho | cal violations |
|------:|-----------------:|--------------:|------:|------:|------:|--------------:|
| 0.0301 | 20.1 | 10.2 | 0.5895 | 0.3036 | -0.1551 | 0 |
| 0.1068 | 23.2 | 9.1 | 0.5735 | 0.3532 | -0.1611 | 0 |
| 0.1936 | 47.9 | 14.0 | 0.5804 | 0.3851 | -0.1539 | 0 |
| 0.2795 | 49.0 | 14.6 | 0.5651 | 0.4003 | -0.1615 | 0 |
| 0.4376 | 52.8 | 14.7 | 0.5377 | 0.4277 | -0.1686 | 0 |
| 0.7014 | 49.1 | 14.6 | 0.5478 | 0.4344 | -0.1672 | 0 |
| 0.9507 | 42.9 | 12.4 | 0.5632 | 0.4462 | -0.1645 | 0 |
| 1.0274 | 38.8 | 11.1 | 0.5643 | 0.4393 | -0.1596 | 0 |
| 1.1988 | 35.3 | 10.9 | 0.5416 | 0.4460 | -0.1638 | 0 |
| 1.4495 | 37.0 | 11.6 | 0.5556 | 0.4432 | -0.1617 | 0 |
| 1.9476 | 35.0 | 13.0 | 0.5651 | 0.4595 | -0.1554 | 0 |
| 2.9452 | 32.7 | 10.4 | 0.5608 | 0.5021 | -0.1530 | 0 |

## Step 2: Surface Fit (with calendar penalty)

| T | max IV err (bps) | RMSE IV (bps) | avg price err (bps) | eta | gamma | rho | phi | eta*(1+\|rho\|) | cal violations | max cal viol (bps) | converged |
|------:|-----------------:|--------------:|--------------------:|------:|------:|------:|------:|---------------:|--------------:|------------------:|:---------:|
| 0.0301 | 20.1 | 10.2 | 8.8 | 0.5895 | 0.3036 | -0.1551 | 4.377 | 0.681 | 0 | 0.0 | yes |
| 0.1068 | 23.2 | 9.1 | 7.7 | 0.5735 | 0.3532 | -0.1611 | 3.981 | 0.666 | 0 | 0.0 | yes |
| 0.1936 | 47.9 | 14.0 | 10.2 | 0.5804 | 0.3851 | -0.1539 | 3.995 | 0.670 | 0 | 0.0 | yes |
| 0.2795 | 49.0 | 14.6 | 10.9 | 0.5651 | 0.4003 | -0.1615 | 3.709 | 0.656 | 0 | 0.0 | yes |
| 0.4376 | 52.8 | 14.7 | 10.4 | 0.5377 | 0.4277 | -0.1686 | 3.394 | 0.628 | 0 | 0.0 | yes |
| 0.7014 | 49.1 | 14.6 | 10.3 | 0.5478 | 0.4344 | -0.1672 | 3.009 | 0.639 | 0 | 0.0 | yes |
| 0.9507 | 42.9 | 12.4 | 8.6 | 0.5632 | 0.4462 | -0.1645 | 2.868 | 0.656 | 0 | 0.0 | yes |
| 1.0274 | 38.8 | 11.1 | 7.7 | 0.5643 | 0.4393 | -0.1596 | 2.740 | 0.654 | 0 | 0.0 | yes |
| 1.1988 | 35.3 | 10.9 | 7.6 | 0.5416 | 0.4460 | -0.1638 | 2.527 | 0.630 | 0 | 0.0 | yes |
| 1.4495 | 37.0 | 11.6 | 8.1 | 0.5556 | 0.4432 | -0.1617 | 2.318 | 0.645 | 0 | 0.0 | yes |
| 1.9476 | 35.0 | 13.0 | 10.2 | 0.5651 | 0.4595 | -0.1554 | 2.100 | 0.653 | 0 | 0.0 | yes |
| 2.9452 | 32.7 | 10.4 | 7.8 | 0.5609 | 0.5021 | -0.1530 | 1.798 | 0.647 | 0 | 0.0 | yes |

## Fit Plots

### T = 0.0301

max err: 20.1 bps | RMSE: 10.2 bps | eta=0.5895, gamma=0.3036, rho=-0.1551 | cal violations: 0

![T=0.0301](plots/fit_surface_T0p030.svg)

### T = 0.1068

max err: 23.2 bps | RMSE: 9.1 bps | eta=0.5735, gamma=0.3532, rho=-0.1611 | cal violations: 0

![T=0.1068](plots/fit_surface_T0p107.svg)

### T = 0.1936

max err: 47.9 bps | RMSE: 14.0 bps | eta=0.5804, gamma=0.3851, rho=-0.1539 | cal violations: 0

![T=0.1936](plots/fit_surface_T0p194.svg)

### T = 0.2795

max err: 49.0 bps | RMSE: 14.6 bps | eta=0.5651, gamma=0.4003, rho=-0.1615 | cal violations: 0

![T=0.2795](plots/fit_surface_T0p280.svg)

### T = 0.4376

max err: 52.8 bps | RMSE: 14.7 bps | eta=0.5377, gamma=0.4277, rho=-0.1686 | cal violations: 0

![T=0.4376](plots/fit_surface_T0p438.svg)

### T = 0.7014

max err: 49.1 bps | RMSE: 14.6 bps | eta=0.5478, gamma=0.4344, rho=-0.1672 | cal violations: 0

![T=0.7014](plots/fit_surface_T0p701.svg)

### T = 0.9507

max err: 42.9 bps | RMSE: 12.4 bps | eta=0.5632, gamma=0.4462, rho=-0.1645 | cal violations: 0

![T=0.9507](plots/fit_surface_T0p951.svg)

### T = 1.0274

max err: 38.8 bps | RMSE: 11.1 bps | eta=0.5643, gamma=0.4393, rho=-0.1596 | cal violations: 0

![T=1.0274](plots/fit_surface_T1p027.svg)

### T = 1.1988

max err: 35.3 bps | RMSE: 10.9 bps | eta=0.5416, gamma=0.4460, rho=-0.1638 | cal violations: 0

![T=1.1988](plots/fit_surface_T1p199.svg)

### T = 1.4495

max err: 37.0 bps | RMSE: 11.6 bps | eta=0.5556, gamma=0.4432, rho=-0.1617 | cal violations: 0

![T=1.4495](plots/fit_surface_T1p450.svg)

### T = 1.9476

max err: 35.0 bps | RMSE: 13.0 bps | eta=0.5651, gamma=0.4595, rho=-0.1554 | cal violations: 0

![T=1.9476](plots/fit_surface_T1p948.svg)

### T = 2.9452

max err: 32.7 bps | RMSE: 10.4 bps | eta=0.5609, gamma=0.5021, rho=-0.1530 | cal violations: 0

![T=2.9452](plots/fit_surface_T2p945.svg)

## Calendar Arbitrage Analysis

Violations checked at 21 k-points from -0.5 to 0.5 (step 0.05).

| T | cal violations | max cal viol (bps) | eta*(1+\|rho\|) |
|------:|--------------:|------------------:|---------------:|
| 0.0301 | 0 | 0.0 | 0.681 |
| 0.1068 | 0 | 0.0 | 0.666 |
| 0.1936 | 0 | 0.0 | 0.670 |
| 0.2795 | 0 | 0.0 | 0.656 |
| 0.4376 | 0 | 0.0 | 0.628 |
| 0.7014 | 0 | 0.0 | 0.639 |
| 0.9507 | 0 | 0.0 | 0.656 |
| 1.0274 | 0 | 0.0 | 0.654 |
| 1.1988 | 0 | 0.0 | 0.630 |
| 1.4495 | 0 | 0.0 | 0.645 |
| 1.9476 | 0 | 0.0 | 0.653 |
| 2.9452 | 0 | 0.0 | 0.647 |
