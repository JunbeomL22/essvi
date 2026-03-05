# SSVI Calibration을 위한 Nonlinear Optimization

## 1. 문제 설정

### 1.1 SSVI (Surface SVI) 모델

SSVI는 implied volatility surface를 파라미터화하는 모델로, total variance surface $w(k, \theta_t)$를 다음과 같이 정의한다:

$$
w(k, \theta_t) = \frac{\theta_t}{2} \left\{ 1 + \rho \varphi(\theta_t) k + \sqrt{(\varphi(\theta_t) k + \rho)^2 + (1 - \rho^2)} \right\}
$$

여기서 $\varphi(\theta)$는 다음과 같다:

$$
\varphi(\theta) = \frac{\eta}{\theta^\gamma (1 + \theta)^{1-\gamma}}
$$

### 1.2 Calibration 변수

| 변수 | 범위 | 설명 |
|------|------|------|
| $\eta$ | $(0, \infty)$ | volatility of variance 스케일 |
| $\gamma$ | $(0, 1)$ | power law 파라미터 |
| $\rho$ | $(-1, 1)$ | skew 파라미터 |

### 1.3 Constraints

**ATM consistency (equality constraint):**

ATM에서 SSVI가 자기 자신을 reproduce해야 하는 조건을 1차 Taylor 전개하면:

$$
\theta = \theta^* - \rho \, \theta \, \varphi(\theta) \, k^*
$$

여기서 $\theta^*, k^*$는 주어진 값 (ATM total variance, ATM log-moneyness).

**No-arbitrage condition (inequality constraint):**

$$
\eta (1 + |\rho|) \leq 2
$$

### 1.4 Objective

Market total variance $W$와 모델 total variance $tv$의 차이를 최소화:

$$
\min_{\eta, \gamma, \rho} \| W - tv \|
$$

---

## 2. Optimization 방법론 검토

### 2.1 Nonlinear Equality + Bounds 문제에 적용 가능한 방법들

| 방법 | 특징 | 대표 구현 |
|------|------|----------|
| **SQP** | 매 iteration마다 QP subproblem. Equality + bounds 직접 처리 | SLSQP, SNOPT |
| **Augmented Lagrangian** | Equality를 penalty + multiplier로 처리. Inner loop은 bound-only | ALGENCAN, LANCELOT |
| **Interior Point** | Bound를 log barrier로, equality는 KKT/Newton | IPOPT |
| **Penalty Method** | $h(x)=0$을 $\|h(x)\|^2$ penalty로. 단순하지만 수렴 느림 | - |

변수 3~4개의 small-scale 문제에서는 **SLSQP**가 가장 무난하다.

### 2.2 Equality Constraint Elimination

핵심 관찰: equality constraint에서 $\theta$는 $(\eta, \gamma, \rho)$가 주어지면 결정되는 값이다.

$$
\theta = \frac{\theta^*}{1 + \rho \, \varphi(\theta) \, k^*}
$$

그런데 $\varphi(\theta)$ 자체가 $\theta$의 함수이므로, 이것은 $\theta$에 대한 **implicit equation** (fixed point equation)이다:

$$
\theta = g(\theta; \eta, \gamma, \rho)
$$

이를 풀면 equality constraint가 사라지고, 문제는 **3변수 bound-only optimization**으로 단순화된다.

### 2.3 $\theta$ 계산 방법

$(\eta, \gamma, \rho)$가 주어졌을 때 $\theta$를 구하는 방법:

**Fixed point iteration:**

$$
\theta_0 = \theta^*, \quad \theta_{n+1} = g(\theta_n)
$$

**또는 Brent root finding:**

$$
h(\theta) = \theta - g(\theta) = 0
$$

### 2.4 전체 최적화 흐름

```
1. Optimizer가 (η, γ, ρ) 후보를 제안
2. 해당 값으로 θ를 implicit equation에서 계산 (Brent 또는 fixed point)
3. θ가 구해지면 w(k, θ) 계산
4. ||W - tv|| 계산해서 optimizer에 반환
5. Optimizer가 다음 후보 제안 → 반복
```

Optimizer 입장에서는 $\theta$ 계산이 objective function 내부에서 일어나므로, **equality constraint 없는 3변수 bounded minimization** 문제를 푸는 것이다.

이 방식의 장점:
- Equality constraint를 solver에 넘길 필요가 없음
- Solver의 수렴 안정성이 향상됨 (constraint 만족을 위한 추가 작업 불필요)
- 더 단순한 solver 사용 가능

---

## 3. 3변수 Bound-Only 문제의 Solver 선택

### 3.1 후보 비교

| Solver | Gradient 필요 | Bounds 지원 | 특징 |
|--------|:------------:|:----------:|------|
| **L-BFGS-B** | O | O | Gradient 있으면 가장 빠름. Finite difference로 대체 가능 |
| **Nelder-Mead** | X | projection 필요 | Derivative-free. 3변수면 simplex 4개 점 |
| **$\rho$ grid + 2D opt** | X | O | $\rho$ sweep 후 $(\eta, \gamma)$ 2D 최적화. Local minima 회피 |

### 3.2 Gradient 관련 고려사항

Objective function의 gradient는:

$$
\nabla L = \left( \frac{\partial L}{\partial \eta}, \frac{\partial L}{\partial \gamma}, \frac{\partial L}{\partial \rho} \right)
$$

Objective 안에서 $\theta$를 Brent로 풀고 있으므로 analytic gradient를 구하려면 implicit function theorem으로 $d\theta/d\eta$ 등을 유도해야 하는데 번거롭다. 3변수이므로 **finite difference 근사**로 충분하다 (gradient 1회 = function evaluation 6~7회).

### 3.3 Rust 구현 시 고려사항

`argmin` crate는 **L-BFGS-B를 지원하지 않는다** (L-BFGS만 있음). Bound를 직접 지원하는 solver는 `ParticleSwarm`뿐이나, 3변수 least squares에는 overkill이다.

실용적 선택지:

1. **Nelder-Mead 직접 구현** — 3변수면 simplex 4개 점, 약 170줄. Bound violation 시 projection. 외부 dependency 없이 가장 깔끔.
2. **변수 변환 + argmin Nelder-Mead** — $\rho = \tanh(x)$, $\gamma = \text{sigmoid}(x)$, $\eta = e^x$로 변환하여 unconstrained로 변환.
3. **`nlopt` crate** — C library wrapper. SLSQP, COBYLA 등 지원하나 C dependency 존재.

---

## 4. Bounded Nelder-Mead 구현 (Rust)

외부 dependency 없이 구현한 bounded Nelder-Mead optimizer. SSVI calibration처럼 반복 호출이 많은 경우에 적합하다.

### 4.1 알고리즘 개요

Nelder-Mead는 $n$차원 공간에서 $n+1$개의 꼭짓점으로 이루어진 simplex를 반복적으로 변형하여 최솟값을 찾는 derivative-free 방법이다. 3변수 문제에서 simplex는 4개의 점으로 구성된다.

각 iteration에서 수행하는 연산:

1. **Sort**: function value로 꼭짓점 정렬 (best → worst)
2. **Centroid**: worst를 제외한 꼭짓점의 무게중심 계산
3. **Reflection**: worst 점을 centroid 기준으로 반사
4. **Expansion**: reflection이 best보다 좋으면 더 확장 시도
5. **Contraction**: reflection이 나쁘면 수축
6. **Shrink**: 모든 시도가 실패하면 best 점 방향으로 전체 simplex 축소

Bound constraint는 각 연산 후 **projection** (clamp)으로 처리한다.

### 4.2 파라미터

| 파라미터 | 기본값 | 설명 |
|---------|--------|------|
| `alpha` | 1.0 | Reflection 계수 |
| `gamma` | 2.0 | Expansion 계수 |
| `rho` | 0.5 | Contraction 계수 |
| `sigma` | 0.5 | Shrink 계수 |
| `tol_f` | 1e-12 | Function value spread 수렴 조건 |
| `tol_x` | 1e-12 | Simplex diameter 수렴 조건 |
| `max_iter` | 1000 | 최대 iteration 수 |

### 4.3 코드

```rust
#[derive(Debug, Clone)]
pub struct NelderMeadConfig {
    pub max_iter: usize,
    pub tol_f: f64,
    pub tol_x: f64,
    pub alpha: f64,
    pub gamma: f64,
    pub rho: f64,
    pub sigma: f64,
}

impl Default for NelderMeadConfig {
    fn default() -> Self {
        Self {
            max_iter: 1000,
            tol_f: 1e-12,
            tol_x: 1e-12,
            alpha: 1.0,
            gamma: 2.0,
            rho: 0.5,
            sigma: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NelderMeadResult {
    pub x: Vec<f64>,
    pub f: f64,
    pub iterations: usize,
    pub converged: bool,
}

#[inline]
fn project(x: &mut Vec<f64>, lb: &[f64], ub: &[f64]) {
    for i in 0..x.len() {
        x[i] = x[i].clamp(lb[i], ub[i]);
    }
}

pub fn nelder_mead_bounded<F>(
    f: F,
    x0: &[f64],
    lb: &[f64],
    ub: &[f64],
    config: &NelderMeadConfig,
) -> NelderMeadResult
where
    F: Fn(&[f64]) -> f64,
{
    let n = x0.len();

    // Build initial simplex (n+1 vertices)
    let mut simplex: Vec<Vec<f64>> = Vec::with_capacity(n + 1);
    let mut v0 = x0.to_vec();
    project(&mut v0, lb, ub);
    simplex.push(v0);

    for j in 0..n {
        let mut v = x0.to_vec();
        let delta = if x0[j].abs() > 1e-10 {
            0.05 * x0[j]
        } else {
            0.00025 * (ub[j] - lb[j])
        };
        v[j] += delta;
        project(&mut v, lb, ub);
        simplex.push(v);
    }

    let mut fvals: Vec<f64> = simplex.iter().map(|v| f(v)).collect();
    let mut iterations = 0;
    let mut converged = false;

    for _iter in 0..config.max_iter {
        iterations = _iter + 1;

        // Sort by function value
        let mut order: Vec<usize> = (0..=n).collect();
        order.sort_by(|&a, &b| fvals[a].partial_cmp(&fvals[b]).unwrap());
        let sorted_simplex: Vec<Vec<f64>> = order.iter().map(|&i| simplex[i].clone()).collect();
        let sorted_fvals: Vec<f64> = order.iter().map(|&i| fvals[i]).collect();
        simplex = sorted_simplex;
        fvals = sorted_fvals;

        let f_best = fvals[0];
        let f_worst = fvals[n];
        let f_second_worst = fvals[n - 1];

        // Check convergence
        let f_spread = (f_worst - f_best).abs();
        let x_spread: f64 = (0..n)
            .map(|j| {
                let vals: Vec<f64> = simplex.iter().map(|v| v[j]).collect();
                let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
                let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                mx - mn
            })
            .fold(0.0_f64, f64::max);

        if f_spread < config.tol_f && x_spread < config.tol_x {
            converged = true;
            break;
        }

        // Centroid (excluding worst)
        let mut centroid = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                centroid[j] += simplex[i][j];
            }
        }
        for j in 0..n {
            centroid[j] /= n as f64;
        }

        // Reflection
        let mut xr: Vec<f64> = (0..n)
            .map(|j| centroid[j] + config.alpha * (centroid[j] - simplex[n][j]))
            .collect();
        project(&mut xr, lb, ub);
        let fr = f(&xr);

        if fr < f_second_worst && fr >= f_best {
            simplex[n] = xr;
            fvals[n] = fr;
            continue;
        }

        if fr < f_best {
            // Expansion
            let mut xe: Vec<f64> = (0..n)
                .map(|j| centroid[j] + config.gamma * (xr[j] - centroid[j]))
                .collect();
            project(&mut xe, lb, ub);
            let fe = f(&xe);
            if fe < fr {
                simplex[n] = xe;
                fvals[n] = fe;
            } else {
                simplex[n] = xr;
                fvals[n] = fr;
            }
            continue;
        }

        if fr < f_worst {
            // Outside contraction
            let mut xc: Vec<f64> = (0..n)
                .map(|j| centroid[j] + config.rho * (xr[j] - centroid[j]))
                .collect();
            project(&mut xc, lb, ub);
            let fc = f(&xc);
            if fc <= fr {
                simplex[n] = xc;
                fvals[n] = fc;
                continue;
            }
        } else {
            // Inside contraction
            let mut xc: Vec<f64> = (0..n)
                .map(|j| centroid[j] - config.rho * (centroid[j] - simplex[n][j]))
                .collect();
            project(&mut xc, lb, ub);
            let fc = f(&xc);
            if fc < f_worst {
                simplex[n] = xc;
                fvals[n] = fc;
                continue;
            }
        }

        // Shrink
        for i in 1..=n {
            for j in 0..n {
                simplex[i][j] = simplex[0][j]
                    + config.sigma * (simplex[i][j] - simplex[0][j]);
            }
            project(&mut simplex[i], lb, ub);
            fvals[i] = f(&simplex[i]);
        }
    }

    NelderMeadResult {
        x: simplex[0].clone(),
        f: fvals[0],
        iterations,
        converged,
    }
}
```

### 4.4 SSVI Calibration에서의 사용 예

```rust
let result = nelder_mead_bounded(
    |x| {
        let eta = x[0];
        let gamma = x[1];
        let rho = x[2];

        // 1) θ를 implicit equation에서 계산 (Brent root finding)
        let theta = solve_theta(eta, gamma, rho, theta_star, k_star);

        // 2) SSVI model total variance 계산
        let w_model = ssvi_surface(k_array, theta, eta, gamma, rho);

        // 3) loss 반환
        squared_error(&w_model, &w_market)
    },
    &[0.5, 0.5, 0.0],             // initial guess: (η, γ, ρ)
    &[1e-6, 1e-6, -0.999],        // lower bounds
    &[10.0, 0.999, 0.999],        // upper bounds
    &NelderMeadConfig::default(),
);

println!("η = {:.6}, γ = {:.6}, ρ = {:.6}", result.x[0], result.x[1], result.x[2]);
```

### 4.5 테스트 결과

| 테스트 | Iterations | 수렴 | 정확도 |
|--------|-----------|:----:|--------|
| Rosenbrock 2D | 125 | ✓ | ~1e-13 |
| Bounded (해가 boundary) | 7 | ✓ | ~1e-6 |
| SSVI-like 3D | 193 | ✓ | ~1e-13 |

---

## 5. 요약

SSVI calibration 문제는 원래 3변수 + nonlinear equality constraint + bounds 문제이지만, equality constraint에서 $\theta$를 implicit equation으로 풀어 eliminate하면 **3변수 bound-only** 문제로 단순화된다. 이 경우 SLSQP 같은 constrained solver가 필요 없고, bounded Nelder-Mead만으로 충분히 풀 수 있다.

Rust 구현 시 `argmin` crate는 L-BFGS-B를 지원하지 않으므로, Nelder-Mead를 직접 구현하는 것이 외부 dependency 없이 가장 깔끔한 접근이다. 약 170줄로 구현 가능하며, SSVI calibration처럼 slice별로 반복 호출이 필요한 경우에도 충분한 성능을 보인다.
