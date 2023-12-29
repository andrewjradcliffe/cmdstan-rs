use crate::builder::Builder;
use crate::translate::Translate;
use std::ffi::OsString;

/// Optimization algorithm. Defaults to `Lbfgs`.
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "algorithm"]
pub enum OptimizeAlgorithm {
    /// BFGS with linesearch
    #[non_exhaustive]
    Bfgs {
        /// Line search step size for first iteration.
        /// Valid values: `0 < init_alpha`.
        /// Defaults to `0.001`.
        #[defaults_to = "crate::consts::INIT_ALPHA"]
        init_alpha: f64,
        /// Convergence tolerance on absolute changes in objective function value.
        /// Valid values: `0 <= tol_obj`.
        /// Defaults to `1e-12`.
        #[defaults_to = "crate::consts::TOL_OBJ"]
        tol_obj: f64,
        /// Convergence tolerance on relative changes in objective function value.
        /// Valid values: `0 <= tol_rel_obj`.
        /// Defaults to `10000.0`.
        #[defaults_to = "crate::consts::TOL_REL_OBJ"]
        tol_rel_obj: f64,
        /// Convergence tolerance on the norm of the gradient.
        /// Valid values: `0 <= tol_grad`.
        /// Defaults to `1e-08`.
        #[defaults_to = "crate::consts::TOL_GRAD"]
        tol_grad: f64,
        /// Convergence tolerance on the relative norm of the gradient.
        /// Valid values: `0 <= tol_rel_grad`.
        /// Defaults to `10000000.0`
        #[defaults_to = "crate::consts::TOL_REL_GRAD"]
        tol_rel_grad: f64,
        /// Convergence tolerance on changes in parameter value.
        /// Valid values: `0 <= tol_param`.
        /// Defaults to `1e-08`.
        #[defaults_to = "crate::consts::TOL_PARAM"]
        tol_param: f64,
    },
    /// LBFGS with linesearch
    #[non_exhaustive]
    Lbfgs {
        /// Line search step size for first iteration.
        /// Valid values: `0 < init_alpha`.
        /// Defaults to `0.001`.
        #[defaults_to = "crate::consts::INIT_ALPHA"]
        init_alpha: f64,
        /// Convergence tolerance on absolute changes in objective function value.
        /// Valid values: `0 <= tol_obj`
        /// Defaults to `1e-12`.
        #[defaults_to = "crate::consts::TOL_OBJ"]
        tol_obj: f64,
        /// Convergence tolerance on relative changes in objective function value.
        /// Valid values: `0 <= tol_rel_obj`.
        /// Defaults to `10000.0`.
        #[defaults_to = "crate::consts::TOL_REL_OBJ"]
        tol_rel_obj: f64,
        /// Convergence tolerance on the norm of the gradient.
        /// Valid values: `0 <= tol_grad`.
        /// Defaults to `1e-08`.
        #[defaults_to = "crate::consts::TOL_GRAD"]
        tol_grad: f64,
        /// Convergence tolerance on the relative norm of the gradient.
        /// Valid values: `0 <= tol_rel_grad`.
        /// Defaults to `10000000.0`.
        #[defaults_to = "crate::consts::TOL_REL_GRAD"]
        tol_rel_grad: f64,
        /// Convergence tolerance on changes in parameter value.
        /// Valid values: `0 <= tol_param`.
        /// Defaults to `1e-08`.
        #[defaults_to = "crate::consts::TOL_PARAM"]
        tol_param: f64,
        /// Amount of history to keep for L-BFGS.
        /// Valid values: `0 < history_size`.
        /// Defaults to `5`.
        #[defaults_to = "crate::consts::HISTORY_SIZE"]
        history_size: i32,
    },
    /// Newton's method
    Newton,
}

impl Default for OptimizeAlgorithm {
    fn default() -> Self {
        LbfgsBuilder::new().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = BfgsBuilder::new()
            .init_alpha(0.1)
            .tol_obj(0.2)
            .tol_rel_obj(0.3)
            .tol_grad(0.4)
            .tol_rel_grad(0.5)
            .tol_param(0.6);
        assert_eq!(x.init_alpha, Some(0.1));
        assert_eq!(x.tol_obj, Some(0.2));
        assert_eq!(x.tol_rel_obj, Some(0.3));
        assert_eq!(x.tol_grad, Some(0.4));
        assert_eq!(x.tol_rel_grad, Some(0.5));
        assert_eq!(x.tol_param, Some(0.6));
        assert_eq!(
            x.build(),
            OptimizeAlgorithm::Bfgs {
                init_alpha: 0.1,
                tol_obj: 0.2,
                tol_rel_obj: 0.3,
                tol_grad: 0.4,
                tol_rel_grad: 0.5,
                tol_param: 0.6,
            }
        );

        let x = LbfgsBuilder::new()
            .init_alpha(0.1)
            .tol_obj(0.2)
            .tol_rel_obj(0.3)
            .tol_grad(0.4)
            .tol_rel_grad(0.5)
            .tol_param(0.6)
            .history_size(100);
        assert_eq!(x.init_alpha, Some(0.1));
        assert_eq!(x.tol_obj, Some(0.2));
        assert_eq!(x.tol_rel_obj, Some(0.3));
        assert_eq!(x.tol_grad, Some(0.4));
        assert_eq!(x.tol_rel_grad, Some(0.5));
        assert_eq!(x.tol_param, Some(0.6));
        assert_eq!(x.history_size, Some(100));
        assert_eq!(
            x.build(),
            OptimizeAlgorithm::Lbfgs {
                init_alpha: 0.1,
                tol_obj: 0.2,
                tol_rel_obj: 0.3,
                tol_grad: 0.4,
                tol_rel_grad: 0.5,
                tol_param: 0.6,
                history_size: 100,
            }
        );
    }

    #[test]
    fn default() {
        let x = LbfgsBuilder::new().build();
        assert_eq!(
            x,
            OptimizeAlgorithm::Lbfgs {
                init_alpha: 0.001,
                tol_obj: 1e-12,
                tol_rel_obj: 10000.0,
                tol_grad: 1e-8,
                tol_rel_grad: 10_000_000.0,
                tol_param: 1e-8,
                history_size: 5,
            }
        );
        let y = OptimizeAlgorithm::default();
        assert_eq!(x, y);

        let x = BfgsBuilder::new().build();
        assert_eq!(
            x,
            OptimizeAlgorithm::Bfgs {
                init_alpha: 0.001,
                tol_obj: 1e-12,
                tol_rel_obj: 10000.0,
                tol_grad: 1e-8,
                tol_rel_grad: 10_000_000.0,
                tol_param: 1e-8,
            }
        );
    }

    #[test]
    fn to_args() {
        let x = LbfgsBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec![
                "algorithm=lbfgs",
                "init_alpha=0.001",
                "tol_obj=0.000000000001",
                "tol_rel_obj=10000",
                "tol_grad=0.00000001",
                "tol_rel_grad=10000000",
                "tol_param=0.00000001",
                "history_size=5",
            ]
        );

        let x = BfgsBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec![
                "algorithm=bfgs",
                "init_alpha=0.001",
                "tol_obj=0.000000000001",
                "tol_rel_obj=10000",
                "tol_grad=0.00000001",
                "tol_rel_grad=10000000",
                "tol_param=0.00000001",
            ]
        );

        let x = OptimizeAlgorithm::Newton;
        assert_eq!(x.to_args(), vec!["algorithm=newton"]);
    }
}
