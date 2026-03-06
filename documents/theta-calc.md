# Newton Iteration for BSVI Surface Inversion

## Problem

Given observed total variance $\theta^*$ and log-moneyness $k^*$, find $\theta$ such that

$$
\theta^* = w(k^*, \theta)
$$

where the BSVI surface is

$$
w(k, \theta) = \frac{\theta}{2}\left\{1 + \rho\varphi(\theta)k + \sqrt{(\varphi(\theta)k + \rho)^2 + (1-\rho^2)}\right\}
$$

and

$$
\varphi(\theta) = \frac{\eta}{\theta^\gamma (1+\theta)^{1-\gamma}}
$$

---

## Derivative of $\varphi$

Writing $\varphi(\theta) = \eta\,\theta^{-\gamma}(1+\theta)^{-(1-\gamma)}$, we get

$$
\varphi'(\theta) = -\frac{(\gamma + \theta)\,\varphi(\theta)}{\theta(1+\theta)}
$$

---

## Derivative of $w$ with respect to $\theta$

Define

$$
S = \sqrt{(\varphi(\theta)k + \rho)^2 + (1-\rho^2)}
$$

Then

$$
\partial_\theta w = \frac{w}{\theta} - \frac{(\gamma + \theta)\,\varphi(\theta)\,k}{2(1+\theta)}\left(\rho + \frac{\varphi(\theta)k + \rho}{S}\right)
$$

---

## Newton Iteration

Starting from $\theta_0 = \theta^*$, iterate:

$$
\theta_{n+1} = \theta_n - \frac{w(k^*, \theta_n) - \theta^*}{\partial_\theta w(k^*, \theta_n)}
$$

Typically converges in 2–3 iterations.

---

## Summary of Evaluation Steps

For each Newton step at $\theta_n$:

1. Compute $\varphi_n = \dfrac{\eta}{\theta_n^\gamma (1+\theta_n)^{1-\gamma}}$

2. Compute $S_n = \sqrt{(\varphi_n k^* + \rho)^2 + (1 - \rho^2)}$

3. Compute $w_n = \dfrac{\theta_n}{2}\left(1 + \rho\varphi_n k^* + S_n\right)$

4. Compute $\partial_\theta w_n = \dfrac{w_n}{\theta_n} - \dfrac{(\gamma + \theta_n)\,\varphi_n\, k^*}{2(1+\theta_n)}\left(\rho + \dfrac{\varphi_n k^* + \rho}{S_n}\right)$

5. Update $\theta_{n+1} = \theta_n - \dfrac{w_n - \theta^*}{\partial_\theta w_n}$