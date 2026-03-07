/// Brent's method for root finding on [a, b].
/// Finds x such that f(x) ≈ 0.

#[derive(Debug, Clone)]
pub struct BrentResult {
    pub root: f64,
    pub iterations: usize,
    pub converged: bool,
}

pub fn brent<F>(f: F, mut a: f64, mut b: f64, tol: f64, max_iter: usize) -> BrentResult
where
    F: Fn(f64) -> f64,
{
    let mut fa = f(a);
    let mut fb = f(b);

    if fa * fb > 0.0 {
        // No sign change — return midpoint as best guess
        return BrentResult {
            root: 0.5 * (a + b),
            iterations: 0,
            converged: false,
        };
    }

    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }

    let mut c = a;
    let mut fc = fa;
    let mut mflag = true;
    let mut d = 0.0;

    for iter in 0..max_iter {
        if fb.abs() < tol || (b - a).abs() < tol {
            return BrentResult {
                root: b,
                iterations: iter + 1,
                converged: true,
            };
        }

        let mut s;
        if (fa - fc).abs() > 1e-15 && (fb - fc).abs() > 1e-15 {
            // Inverse quadratic interpolation
            s = a * fb * fc / ((fa - fb) * (fa - fc))
                + b * fa * fc / ((fb - fa) * (fb - fc))
                + c * fa * fb / ((fc - fa) * (fc - fb));
        } else {
            // Secant method
            s = b - fb * (b - a) / (fb - fa);
        }

        let cond1 = {
            let lo = (3.0 * a + b) / 4.0;
            let hi = b;
            let (lo, hi) = if lo < hi { (lo, hi) } else { (hi, lo) };
            s < lo || s > hi
        };
        let cond2 = mflag && (s - b).abs() >= (b - c).abs() / 2.0;
        let cond3 = !mflag && (s - b).abs() >= (c - d).abs() / 2.0;
        let cond4 = mflag && (b - c).abs() < tol;
        let cond5 = !mflag && (c - d).abs() < tol;

        if cond1 || cond2 || cond3 || cond4 || cond5 {
            s = (a + b) / 2.0;
            mflag = true;
        } else {
            mflag = false;
        }

        let fs = f(s);
        d = c;
        c = b;
        fc = fb;

        if fa * fs < 0.0 {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }

        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }

    BrentResult {
        root: b,
        iterations: max_iter,
        converged: false,
    }
}
