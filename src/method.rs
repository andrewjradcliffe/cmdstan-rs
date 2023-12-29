use crate::diagnose::*;
use crate::generate_quantities::*;
use crate::laplace::*;
use crate::log_prob::*;
use crate::optimize::*;
use crate::pathfinder::*;
use crate::sample::*;
use crate::translate::Translate;
use crate::variational::*;
use std::ffi::OsString;

/// Analysis method. Defaults to [`Self::Sample`].
#[derive(Debug, PartialEq, Clone, Translate)]
#[non_exhaustive]
#[declare = "method"]
pub enum Method {
    /// Bayesian inference with Markov Chain Monte Carlo. Use
    /// [`SampleBuilder`] for parameterized construction with optional defaults.
    #[non_exhaustive]
    Sample {
        /// Number of warmup iterations.
        /// Valid values: `0 <= num_samples`.
        /// Defaults to `1000`.
        num_samples: i32,
        /// Number of warmup iterations
        /// Valid values: `0 <= warmup`.
        /// Defaults to `1000`.
        num_warmup: i32,
        /// Stream warmup samples to output?
        /// Defaults to `false`.
        save_warmup: bool,
        /// Period between saved samples.
        /// Valid values: `0 < thin`.
        /// Defaults to `1`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        thin: i32,
        /// Warmup Adaptation
        adapt: SampleAdapt,
        /// Sampling algorithm. Defaults to [`SampleAlgorithm::Hmc`].
        algorithm: SampleAlgorithm,
        /// Number of chains.
        /// Valid values: `num_chains > 0`.
        /// Defaults to `1`.
        num_chains: i32,
    },
    /// Point estimation.
    /// Use [`OptimizeBuilder`] for parameterized construction with optional defaults.
    #[non_exhaustive]
    Optimize {
        /// Optimization algorithm. Defaults to [`OptimizeAlgorithm::Lbfgs`].
        algorithm: OptimizeAlgorithm,
        /// When true, include change-of-variables adjustment for
        /// constraining parameter transforms.
        /// Defaults to `false`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        jacobian: bool,
        /// Total number of iterations.
        /// Valid values: `0 < iter`.
        /// Defaults to `2000`.
        iter: i32,
        /// Stream optimization progress to output?
        /// Defaults to `false`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        save_iterations: bool,
    },
    /// Variational inference. Use [`VariationalBuilder`] for
    /// parameterized construction with optional defaults.
    #[non_exhaustive]
    Variational {
        /// Variational inference algorithm.
        /// Defaults to [`VariationalAlgorithm::MeanField`].
        algorithm: VariationalAlgorithm,
        /// Maximum number of ADVI iterations.
        /// Valid values: `0 < iter`.
        /// Defaults to `10000`.
        iter: i32,
        /// Number of Monte Carlo draws for computing the gradient.
        /// Valid values: `0 < grad_samples`.
        /// Defaults to `1`.
        grad_samples: i32,
        /// Number of Monte Carlo draws for estimate of ELBO.
        /// Valid values: `0 < elbo_samples`.
        /// Defaults to `100`.
        elbo_samples: i32,
        /// Stepsize scaling parameter.
        /// Valid values: `0 < eta`.
        /// Defaults to `1.0`.
        eta: f64,
        /// Eta Adaptation for Variational Inference.
        adapt: VariationalAdapt,
        /// Relative tolerance parameter for convergence.
        /// Valid values: `0 <= tol_rel_obj`.
        /// Defaults to `0.01`.
        tol_rel_obj: f64,
        /// Number of iterations between ELBO evaluations.
        /// Valid values: `0 < eval_elbo`.
        /// Defaults to `100`.
        eval_elbo: i32,
        /// Number of approximate posterior output draws to save.
        /// Valid values: `0 < output_samples`.
        /// Defaults to `1000`.
        output_samples: i32,
    },
    /// Model diagnostics. Use [`DiagnoseBuilder`] for construction
    /// with defaults.
    #[non_exhaustive]
    Diagnose {
        /// Diagnostic test. Defaults to [`DiagnoseTest::Gradient`].
        test: DiagnoseTest,
    },
    /// Generate quantities of interest
    #[non_exhaustive]
    #[declare = "generate_quantities"]
    GenerateQuantities {
        /// Input file of sample of fitted parameter values for model conditioned on data.
        /// Valid values: Path to existing file.
        /// Defaults to `""`.
        fitted_params: OsString,
    },
    /// Pathfinder algorithm. Use [`PathfinderBuilder`] for
    /// construction with defaults.
    #[non_exhaustive]
    Pathfinder {
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
        /// Defaults to `10000000.0`.
        tol_rel_grad: f64,
        /// Convergence tolerance on changes in parameter value.
        /// Valid values: `0 <= tol_param`.
        /// Defaults to `1e-08`
        tol_param: f64,
        /// Amount of history to keep for L-BFGS.
        /// Valid values: `0 < history_size`.
        /// Defaults to `5`.
        history_size: i32,
        /// Number of draws from PSIS sample.
        /// Valid values: `0 < num_psis_draws`.
        /// Defaults to `1000`.
        num_psis_draws: i32,
        /// Number of single pathfinders.
        /// Valid values: `0 < num_paths`.
        /// Defaults to `4`.
        num_paths: i32,
        /// Output single-path pathfinder draws as CSV.
        /// Defaults to `false`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        save_single_paths: bool,
        /// Maximum number of LBFGS iterations.
        /// Valid values: `0 < max_lbfgs_iters`.
        /// Defaults to `1000`.
        max_lbfgs_iters: i32,
        /// Number of approximate posterior draws.
        /// Valid values: `0 < num_draws`.
        /// Defaults to `1000`.
        num_draws: i32,
        /// Number of Monte Carlo draws to evaluate ELBO.
        /// Valid values: `0 < num_elbo_draws`.
        /// Defaults to `25`.
        num_elbo_draws: i32,
    },
    /// Return the log density up to a constant and its gradients, given supplied parameters.
    /// Use [`LogProbBuilder`] for parameterized construction with optional defaults.
    #[non_exhaustive]
    #[declare = "log_prob"]
    LogProb {
        /// Input file (JSON or R dump) of parameter values on unconstrained scale.
        /// Valid values: Path to existing file.
        /// Defaults to `""`.
        unconstrained_params: OsString,
        /// Input file (JSON or R dump) of parameter values on constrained scale.
        /// Valid values: Path to existing file.
        /// Defaults to `""`.
        constrained_params: OsString,
        /// When true, include change-of-variables adjustment for
        /// constraining parameter transforms.
        /// Defaults to `true`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        jacobian: bool,
    },
    /// Sample from a Laplace approximation.
    /// Use [`LaplaceBuilder`] for parameterized construction with optional defaults.
    #[non_exhaustive]
    Laplace {
        /// A specification of a mode on the constrained scale for all
        /// model parameters, either in JSON or CSV format.
        /// Valid values: Path to existing file.
        /// Defaults to `""`.
        mode: OsString,
        /// When true, include change-of-variables adjustment for
        /// constraining parameter transforms.
        /// Defaults to `true`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        jacobian: bool,
        /// Number of draws from the laplace approximation.
        /// Valid values: `0 <= draws`.
        /// Defaults to `1000`.
        draws: i32,
    },
}

impl Default for Method {
    fn default() -> Self {
        SampleBuilder::new().build()
    }
}
macro_rules! from_impl {
    ($T:ident) => {
        impl From<$T> for Method {
            fn from(x: $T) -> Self {
                x.build()
            }
        }
    };
}
from_impl!(SampleBuilder);
from_impl!(OptimizeBuilder);
from_impl!(VariationalBuilder);
from_impl!(DiagnoseBuilder);
from_impl!(GenerateQuantitiesBuilder);
from_impl!(PathfinderBuilder);
from_impl!(LogProbBuilder);
from_impl!(LaplaceBuilder);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_to_args() {
        let x = SampleBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec![
                "method=sample",
                "num_samples=1000",
                "num_warmup=1000",
                "save_warmup=0",
                "thin=1",
                "adapt",
                "engaged=1",
                "gamma=0.05",
                "delta=0.8",
                "kappa=0.75",
                "t0=10",
                "init_buffer=75",
                "term_buffer=50",
                "window=25",
                "algorithm=hmc",
                "engine=nuts",
                "max_depth=10",
                "metric=diag_e",
                "metric_file=",
                "stepsize=1",
                "stepsize_jitter=0",
                "num_chains=1"
            ]
        );
    }

    #[test]
    fn optimize_to_args() {
        let x = OptimizeBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec![
                "method=optimize",
                "algorithm=lbfgs",
                "init_alpha=0.001",
                "tol_obj=0.000000000001",
                "tol_rel_obj=10000",
                "tol_grad=0.00000001",
                "tol_rel_grad=10000000",
                "tol_param=0.00000001",
                "history_size=5",
                "jacobian=0",
                "iter=2000",
                "save_iterations=0"
            ]
        );
    }

    #[test]
    fn variational_to_args() {
        let x = VariationalBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec![
                "method=variational",
                "algorithm=meanfield",
                "iter=10000",
                "grad_samples=1",
                "elbo_samples=100",
                "eta=1",
                "adapt",
                "engaged=1",
                "iter=50",
                "tol_rel_obj=0.01",
                "eval_elbo=100",
                "output_samples=1000"
            ]
        );
    }

    #[test]
    fn diagnose_to_args() {
        let x = DiagnoseBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec![
                "method=diagnose",
                "test=gradient",
                "epsilon=0.000001",
                "error=0.000001"
            ]
        );
    }

    #[test]
    fn generate_quantities_to_args() {
        let x = GenerateQuantitiesBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec!["method=generate_quantities", "fitted_params="]
        );
    }

    #[test]
    fn pathfinder_to_args() {
        let x = PathfinderBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec![
                "method=pathfinder",
                "init_alpha=0.001",
                "tol_obj=0.000000000001",
                "tol_rel_obj=10000",
                "tol_grad=0.00000001",
                "tol_rel_grad=10000000",
                "tol_param=0.00000001",
                "history_size=5",
                "num_psis_draws=1000",
                "num_paths=4",
                "save_single_paths=0",
                "max_lbfgs_iters=1000",
                "num_draws=1000",
                "num_elbo_draws=25"
            ]
        );
    }

    #[test]
    fn log_prob_to_args() {
        let x = LogProbBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec![
                "method=log_prob",
                "unconstrained_params=",
                "constrained_params=",
                "jacobian=1"
            ]
        );
    }

    #[test]
    fn laplace_to_args() {
        let x = LaplaceBuilder::new().build();
        assert_eq!(
            x.to_args(),
            vec!["method=laplace", "mode=", "jacobian=1", "draws=1000"]
        );
    }
}
