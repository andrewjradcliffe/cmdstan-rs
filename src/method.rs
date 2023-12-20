use crate::diagnose::*;
use crate::generate_quantities::*;
use crate::laplace::*;
use crate::log_prob::*;
use crate::optimize::*;
use crate::pathfinder::*;
use crate::sample::*;
use crate::variational::*;
use std::ffi::OsString;
// use std::fmt::Write;

/// Analysis method. Defaults to [`Self::Sample`].
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
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
    #[non_exhaustive]
    /// Model diagnostics. Use [`DiagnoseBuilder`] for construction
    /// with defaults.
    Diagnose {
        /// Diagnostic test. Defaults to [`DiagnoseTest::Gradient`].
        test: DiagnoseTest,
    },
    #[non_exhaustive]
    /// Generate quantities of interest
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
use Method::*;
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

impl Method {
    pub fn command_fragment(&self) -> Vec<OsString> {
        match &self {
            Sample {
                num_samples,
                num_warmup,
                save_warmup,
                thin,
                adapt,
                algorithm,
                num_chains,
            } => {
                let mut adapt = adapt.command_fragment();
                let mut algorithm = algorithm.command_fragment();
                let mut v = Vec::with_capacity(6 + adapt.len() + algorithm.len());
                v.push("method=sample".into());
                v.push(format!("num_samples={}", num_samples).into());
                v.push(format!("num_warmup={}", num_warmup).into());
                v.push(format!("save_warmup={}", *save_warmup as u8).into());
                v.push(format!("thin={}", thin).into());
                v.append(&mut adapt);
                v.append(&mut algorithm);
                v.push(format!("num_chains={}", num_chains).into());
                v
            }
            Optimize {
                algorithm,
                jacobian,
                iter,
                save_iterations,
            } => {
                let mut algorithm = algorithm.command_fragment();
                let mut v = Vec::with_capacity(4 + algorithm.len());
                v.push("method=optimize".into());
                v.append(&mut algorithm);
                v.push(format!("jacobian={}", *jacobian as u8).into());
                v.push(format!("iter={}", iter).into());
                v.push(format!("save_iterations={}", *save_iterations as u8).into());
                v
            }
            Variational {
                algorithm,
                iter,
                grad_samples,
                elbo_samples,
                eta,
                adapt,
                tol_rel_obj,
                eval_elbo,
                output_samples,
            } => {
                let mut algorithm = algorithm.command_fragment();
                let mut adapt = adapt.command_fragment();
                let mut v = Vec::with_capacity(8 + algorithm.len() + adapt.len());
                v.push("method=variational".into());
                v.append(&mut algorithm);
                v.push(format!("iter={}", iter).into());
                v.push(format!("grad_samples={}", grad_samples).into());
                v.push(format!("elbo_samples={}", elbo_samples).into());
                v.push(format!("eta={}", eta).into());
                v.append(&mut adapt);
                v.push(format!("tol_rel_obj={}", tol_rel_obj).into());
                v.push(format!("eval_elbo={}", eval_elbo).into());
                v.push(format!("output_samples={}", output_samples).into());
                v
            }
            Diagnose { test } => {
                let mut test = test.command_fragment();
                let mut v = Vec::with_capacity(1 + test.len());
                v.push("method=diagnose".into());
                v.append(&mut test);
                v
            }
            GenerateQuantities { fitted_params } => {
                let mut v = Vec::with_capacity(2);
                v.push("method=generate_quantities".into());
                let mut s = OsString::with_capacity(14 + fitted_params.len());
                s.push("fitted_params=");
                s.push(fitted_params);
                v.push(s);
                v
            }
            Pathfinder {
                init_alpha,
                tol_obj,
                tol_rel_obj,
                tol_grad,
                tol_rel_grad,
                tol_param,
                history_size,
                num_psis_draws,
                num_paths,
                save_single_paths,
                max_lbfgs_iters,
                num_draws,
                num_elbo_draws,
            } => {
                let mut v = Vec::with_capacity(14);
                v.push("method=pathfinder".into());
                v.push(format!("init_alpha={}", init_alpha).into());
                v.push(format!("tol_obj={}", tol_obj).into());
                v.push(format!("tol_rel_obj={}", tol_rel_obj).into());
                v.push(format!("tol_grad={}", tol_grad).into());
                v.push(format!("tol_rel_grad={}", tol_rel_grad).into());
                v.push(format!("tol_param={}", tol_param).into());
                v.push(format!("history_size={}", history_size).into());
                v.push(format!("num_psis_draws={}", num_psis_draws).into());
                v.push(format!("num_paths={}", num_paths).into());
                v.push(format!("save_single_paths={}", *save_single_paths as u8).into());
                v.push(format!("max_lbfgs_iters={}", max_lbfgs_iters).into());
                v.push(format!("num_draws={}", num_draws).into());
                v.push(format!("num_elbo_draws={}", num_elbo_draws).into());
                v
            }
            LogProb {
                unconstrained_params,
                constrained_params,
                jacobian,
            } => {
                let mut v = Vec::with_capacity(4);
                v.push("method=log_prob".into());
                if !unconstrained_params.is_empty() {
                    let mut s = OsString::with_capacity(21 + unconstrained_params.len());
                    s.push("unconstrained_params=");
                    s.push(unconstrained_params);
                    v.push(s);
                }
                if !constrained_params.is_empty() {
                    let mut s = OsString::with_capacity(19 + constrained_params.len());
                    s.push("constrained_params=");
                    s.push(constrained_params);
                    v.push(s);
                }
                v.push(format!("jacobian={}", *jacobian as u8).into());
                v
            }
            Laplace {
                mode,
                jacobian,
                draws,
            } => {
                let mut v = Vec::with_capacity(4);
                v.push("method=laplace".into());
                if !mode.is_empty() {
                    let mut s = OsString::with_capacity(5 + mode.len());
                    s.push("mode=");
                    s.push(mode);
                    v.push(s);
                }
                v.push(format!("jacobian={}", *jacobian as u8).into());
                v.push(format!("draws={}", draws).into());
                v
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_command_fragment() {
        let x = SampleBuilder::new().build();
        assert_eq!(
            x.command_fragment(),
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
                "stepsize=1",
                "stepsize_jitter=0",
                "num_chains=1"
            ]
        );
    }

    #[test]
    fn optimize_command_fragment() {
        let x = OptimizeBuilder::new().build();
        assert_eq!(
            x.command_fragment(),
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
    fn variational_command_fragment() {
        let x = VariationalBuilder::new().build();
        assert_eq!(
            x.command_fragment(),
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
    fn diagnose_command_fragment() {
        let x = DiagnoseBuilder::new().build();
        assert_eq!(
            x.command_fragment(),
            vec![
                "method=diagnose",
                "test=gradient",
                "epsilon=0.000001",
                "error=0.000001"
            ]
        );
    }

    #[test]
    fn generate_quantities_command_fragment() {
        let x = GenerateQuantitiesBuilder::new().build();
        assert_eq!(
            x.command_fragment(),
            vec!["method=generate_quantities", "fitted_params="]
        );
    }

    #[test]
    fn pathfinder_command_fragment() {
        let x = PathfinderBuilder::new().build();
        assert_eq!(
            x.command_fragment(),
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
    fn log_prob_command_fragment() {
        let x = LogProbBuilder::new().build();
        assert_eq!(x.command_fragment(), vec!["method=log_prob", "jacobian=1"]);
    }

    #[test]
    fn laplace_command_fragment() {
        let x = LaplaceBuilder::new().build();
        assert_eq!(
            x.command_fragment(),
            vec!["method=laplace", "jacobian=1", "draws=1000"]
        );
    }
}
