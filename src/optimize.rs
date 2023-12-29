use crate::consts::{
    HISTORY_SIZE, INIT_ALPHA, TOL_GRAD, TOL_OBJ, TOL_PARAM, TOL_REL_GRAD, TOL_REL_OBJ,
};
use crate::method::Method;
use crate::translate::Translate;
use std::ffi::OsString;

/// Options builder for [`Method::Optimize`].
/// For any option left unspecified, the default value indicated
/// on `Method::Optimize` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct OptimizeBuilder {
    algorithm: Option<OptimizeAlgorithm>,
    jacobian: Option<bool>,
    iter: Option<i32>,
    save_iterations: Option<bool>,
}

impl OptimizeBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            algorithm: None,
            jacobian: None,
            iter: None,
            save_iterations: None,
        }
    }
    insert_into_field!(algorithm, OptimizeAlgorithm);
    insert_field!(jacobian, bool);
    insert_field!(iter, i32);
    insert_field!(save_iterations, bool);
    /// Build the `Method::Optimize` instance.
    pub fn build(self) -> Method {
        let algorithm = self.algorithm.unwrap_or_default();
        let jacobian = self.jacobian.unwrap_or(false);
        let iter = self.iter.unwrap_or(2000);
        let save_iterations = self.save_iterations.unwrap_or(false);
        Method::Optimize {
            algorithm,
            jacobian,
            iter,
            save_iterations,
        }
    }
}

impl Default for OptimizeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization algorithm. Defaults to `Lbfgs`.
#[derive(Debug, PartialEq, Clone, Translate)]
#[non_exhaustive]
#[declare = "algorithm"]
pub enum OptimizeAlgorithm {
    /// BFGS with linesearch
    #[non_exhaustive]
    Bfgs {
        /// Line search step size for first iteration.
        /// Valid values: `0 < init_alpha`.
        /// Defaults to `0.001`.
        init_alpha: f64,
        /// Convergence tolerance on absolute changes in objective function value.
        /// Valid values: `0 <= tol_obj`.
        /// Defaults to `1e-12`.
        tol_obj: f64,
        /// Convergence tolerance on relative changes in objective function value.
        /// Valid values: `0 <= tol_rel_obj`.
        /// Defaults to `10000.0`.
        tol_rel_obj: f64,
        /// Convergence tolerance on the norm of the gradient.
        /// Valid values: `0 <= tol_grad`.
        /// Defaults to `1e-08`.
        tol_grad: f64,
        /// Convergence tolerance on the relative norm of the gradient.
        /// Valid values: `0 <= tol_rel_grad`.
        /// Defaults to `10000000.0`
        tol_rel_grad: f64,
        /// Convergence tolerance on changes in parameter value.
        /// Valid values: `0 <= tol_param`.
        /// Defaults to `1e-08`.
        tol_param: f64,
    },
    /// LBFGS with linesearch
    #[non_exhaustive]
    Lbfgs {
        /// Line search step size for first iteration.
        /// Valid values: `0 < init_alpha`.
        /// Defaults to `0.001`.
        init_alpha: f64,
        /// Convergence tolerance on absolute changes in objective function value.
        /// Valid values: `0 <= tol_obj`
        /// Defaults to `1e-12`.
        tol_obj: f64,
        /// Convergence tolerance on relative changes in objective function value.
        /// Valid values: `0 <= tol_rel_obj`.
        /// Defaults to `10000.0`.
        tol_rel_obj: f64,
        /// Convergence tolerance on the norm of the gradient.
        /// Valid values: `0 <= tol_grad`.
        /// Defaults to `1e-08`.
        tol_grad: f64,
        /// Convergence tolerance on the relative norm of the gradient.
        /// Valid values: `0 <= tol_rel_grad`.
        /// Defaults to `10000000.0`.
        tol_rel_grad: f64,
        /// Convergence tolerance on changes in parameter value.
        /// Valid values: `0 <= tol_param`.
        /// Defaults to `1e-08`.
        tol_param: f64,
        /// Amount of history to keep for L-BFGS.
        /// Valid values: `0 < history_size`.
        /// Defaults to `5`.
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

/// Options builder for [`OptimizeAlgorithm::Bfgs`].
/// For any option left unspecified, the default value indicated
/// on `OptimizeAlgorithm::Bfgs` will be supplied.
#[derive(Debug, Clone, PartialEq)]
pub struct BfgsBuilder {
    init_alpha: Option<f64>,
    tol_obj: Option<f64>,
    tol_rel_obj: Option<f64>,
    tol_grad: Option<f64>,
    tol_rel_grad: Option<f64>,
    tol_param: Option<f64>,
}
impl BfgsBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            init_alpha: None,
            tol_obj: None,
            tol_rel_obj: None,
            tol_grad: None,
            tol_rel_grad: None,
            tol_param: None,
        }
    }
    insert_field!(init_alpha, f64);
    insert_field!(tol_obj, f64);
    insert_field!(tol_rel_obj, f64);
    insert_field!(tol_grad, f64);
    insert_field!(tol_rel_grad, f64);
    insert_field!(tol_param, f64);
    /// Build the `OptimizeAlgorithm::Bfgs` instance.
    pub fn build(self) -> OptimizeAlgorithm {
        let init_alpha = self.init_alpha.unwrap_or(INIT_ALPHA);
        let tol_obj = self.tol_obj.unwrap_or(TOL_OBJ);
        let tol_rel_obj = self.tol_rel_obj.unwrap_or(TOL_REL_OBJ);
        let tol_grad = self.tol_grad.unwrap_or(TOL_GRAD);
        let tol_rel_grad = self.tol_rel_grad.unwrap_or(TOL_REL_GRAD);
        let tol_param = self.tol_param.unwrap_or(TOL_PARAM);
        OptimizeAlgorithm::Bfgs {
            init_alpha,
            tol_obj,
            tol_rel_obj,
            tol_grad,
            tol_rel_grad,
            tol_param,
        }
    }
}

impl From<BfgsBuilder> for OptimizeAlgorithm {
    fn from(x: BfgsBuilder) -> Self {
        x.build()
    }
}

impl Default for BfgsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Options builder for [`OptimizeAlgorithm::Lbfgs`].
/// For any option left unspecified, the default value indicated
/// on `OptimizeAlgorithm::Lbfgs` will be supplied.
#[derive(Debug, Clone, PartialEq)]
pub struct LbfgsBuilder {
    init_alpha: Option<f64>,
    tol_obj: Option<f64>,
    tol_rel_obj: Option<f64>,
    tol_grad: Option<f64>,
    tol_rel_grad: Option<f64>,
    tol_param: Option<f64>,
    history_size: Option<i32>,
}
impl LbfgsBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            init_alpha: None,
            tol_obj: None,
            tol_rel_obj: None,
            tol_grad: None,
            tol_rel_grad: None,
            tol_param: None,
            history_size: None,
        }
    }
    insert_field!(init_alpha, f64);
    insert_field!(tol_obj, f64);
    insert_field!(tol_rel_obj, f64);
    insert_field!(tol_grad, f64);
    insert_field!(tol_rel_grad, f64);
    insert_field!(tol_param, f64);
    insert_field!(history_size, i32);
    /// Build the `OptimizeAlgorithm::Lbfgs` instance.
    pub fn build(self) -> OptimizeAlgorithm {
        let init_alpha = self.init_alpha.unwrap_or(INIT_ALPHA);
        let tol_obj = self.tol_obj.unwrap_or(TOL_OBJ);
        let tol_rel_obj = self.tol_rel_obj.unwrap_or(TOL_REL_OBJ);
        let tol_grad = self.tol_grad.unwrap_or(TOL_GRAD);
        let tol_rel_grad = self.tol_rel_grad.unwrap_or(TOL_REL_GRAD);
        let tol_param = self.tol_param.unwrap_or(TOL_PARAM);
        let history_size = self.history_size.unwrap_or(HISTORY_SIZE);
        OptimizeAlgorithm::Lbfgs {
            init_alpha,
            tol_obj,
            tol_rel_obj,
            tol_grad,
            tol_rel_grad,
            tol_param,
            history_size,
        }
    }
}

impl From<LbfgsBuilder> for OptimizeAlgorithm {
    fn from(x: LbfgsBuilder) -> Self {
        x.build()
    }
}

impl Default for LbfgsBuilder {
    fn default() -> Self {
        Self::new()
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

        let x = OptimizeBuilder::new()
            .algorithm(OptimizeAlgorithm::default())
            .jacobian(true)
            .iter(1)
            .save_iterations(true);
        assert_eq!(x.algorithm, Some(OptimizeAlgorithm::default()));
        assert_eq!(x.jacobian, Some(true));
        assert_eq!(x.iter, Some(1));
        assert_eq!(x.save_iterations, Some(true));
        assert_eq!(
            x.build(),
            Method::Optimize {
                algorithm: OptimizeAlgorithm::default(),
                jacobian: true,
                iter: 1,
                save_iterations: true,
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
