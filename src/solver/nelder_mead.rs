/// Bounded Nelder-Mead optimizer (derivative-free).

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
fn project(x: &mut [f64], lb: &[f64], ub: &[f64]) {
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

    for iter in 0..config.max_iter {
        iterations = iter + 1;

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
                let mn = simplex.iter().map(|v| v[j]).fold(f64::INFINITY, f64::min);
                let mx = simplex
                    .iter()
                    .map(|v| v[j])
                    .fold(f64::NEG_INFINITY, f64::max);
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
        for c in centroid.iter_mut() {
            *c /= n as f64;
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
                simplex[i][j] =
                    simplex[0][j] + config.sigma * (simplex[i][j] - simplex[0][j]);
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
