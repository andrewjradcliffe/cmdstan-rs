/// Optimization algorithm
/// Valid values: bfgs, lbfgs, newton
/// Defaults to lbfgs
#[derive(Debug)]
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
