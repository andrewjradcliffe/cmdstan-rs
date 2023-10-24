use std::fmt::Write;

/// Optimization algorithm
/// Valid values: bfgs, lbfgs, newton
/// Defaults to lbfgs
#[derive(Debug, PartialEq)]
pub enum OptimizationAlgorithm {
    /// BFGS with linesearch
    Bfgs {
        /// Line search step size for first iteration
        /// Valid values: 0 < init_alpha
        /// Defaults to 0.001
        init_alpha: f64,
        /// Convergence tolerance on absolute changes in objective function value
        /// Valid values: 0 <= tol
        /// Defaults to 9.9999999999999998e-13
        tol_obj: f64,
        /// Convergence tolerance on relative changes in objective function value
        /// Valid values: 0 <= tol
        /// Defaults to 10000
        tol_rel_obj: f64,
        /// Convergence tolerance on the norm of the gradient
        /// Valid values: 0 <= tol
        /// Defaults to 1e-08
        tol_grad: f64,
        /// Convergence tolerance on the relative norm of the gradient
        /// Valid values: 0 <= tol
        /// Defaults to 10000000
        tol_rel_grad: f64,
        /// Convergence tolerance on changes in parameter value
        /// Valid values: 0 <= tol
        /// Defaults to 1e-08
        tol_param: f64,
    },
    /// LBFGS with linesearch
    Lbfgs {
        /// Line search step size for first iteration
        /// Valid values: 0 < init_alpha
        /// Defaults to 0.001
        init_alpha: f64,
        /// Convergence tolerance on absolute changes in objective function value
        /// Valid values: 0 <= tol
        /// Defaults to 9.9999999999999998e-13
        tol_obj: f64,
        /// Convergence tolerance on relative changes in objective function value
        /// Valid values: 0 <= tol
        /// Defaults to 10000
        tol_rel_obj: f64,
        /// Convergence tolerance on the norm of the gradient
        /// Valid values: 0 <= tol
        /// Defaults to 1e-08
        tol_grad: f64,
        /// Convergence tolerance on the relative norm of the gradient
        /// Valid values: 0 <= tol
        /// Defaults to 10000000
        tol_rel_grad: f64,
        /// Convergence tolerance on changes in parameter value
        /// Valid values: 0 <= tol
        /// Defaults to 1e-08
        tol_param: f64,
        /// Amount of history to keep for L-BFGS
        /// Valid values: 0 < history_size
        /// Defaults to 5
        history_size: i32,
    },
    /// Newton's method
    Newton,
}

impl Default for OptimizationAlgorithm {
    fn default() -> Self {
        OptimizationAlgorithm::Lbfgs {
            init_alpha: 0.001,
            tol_obj: 9.9999999999999998e-13,
            tol_rel_obj: 10000.0,
            tol_grad: 1e-8,
            tol_rel_grad: 10_000_000.0,
            tol_param: 1e-8,
            history_size: 5,
        }
    }
}
use OptimizationAlgorithm::*;

impl OptimizationAlgorithm {
    pub fn command_fragment(&self) -> String {
        match &self {
            Bfgs {
                init_alpha,
                tol_obj,
                tol_rel_obj,
                tol_grad,
                tol_rel_grad,
                tol_param,
            } => {
                let mut s = String::from("algorithm=bfgs");
                write!(&mut s, " init_alpha={}", init_alpha).unwrap();
                write!(&mut s, " tol_obj={}", tol_obj).unwrap();
                write!(&mut s, " tol_rel_obj={}", tol_rel_obj).unwrap();
                write!(&mut s, " tol_grad={}", tol_grad).unwrap();
                write!(&mut s, " tol_rel_grad={}", tol_rel_grad).unwrap();
                write!(&mut s, " tol_param={}", tol_param).unwrap();
                s
            }
            Lbfgs {
                init_alpha,
                tol_obj,
                tol_rel_obj,
                tol_grad,
                tol_rel_grad,
                tol_param,
                history_size,
            } => {
                let mut s = String::from("algorithm=lbfgs");
                write!(&mut s, " init_alpha={}", init_alpha).unwrap();
                write!(&mut s, " tol_obj={}", tol_obj).unwrap();
                write!(&mut s, " tol_rel_obj={}", tol_rel_obj).unwrap();
                write!(&mut s, " tol_grad={}", tol_grad).unwrap();
                write!(&mut s, " tol_rel_grad={}", tol_rel_grad).unwrap();
                write!(&mut s, " tol_param={}", tol_param).unwrap();
                write!(&mut s, " history_size={}", history_size).unwrap();
                s
            }
            Newton => "algorithm=newton".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let x = OptimizationAlgorithm::default();
        assert_eq!(
            x,
            OptimizationAlgorithm::Lbfgs {
                init_alpha: 0.001,
                tol_obj: 9.9999999999999998e-13,
                tol_rel_obj: 10000.0,
                tol_grad: 1e-8,
                tol_rel_grad: 10_000_000.0,
                tol_param: 1e-8,
                history_size: 5,
            }
        );
    }

    #[test]
    fn command_fragment() {
        let x = OptimizationAlgorithm::default();
        assert_eq!(x.command_fragment(), "algorithm=lbfgs init_alpha=0.001 tol_obj=0.000000000001 tol_rel_obj=10000 tol_grad=0.00000001 tol_rel_grad=10000000 tol_param=0.00000001 history_size=5");
    }
}
