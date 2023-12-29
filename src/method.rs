/*! Module for specifying inference algorithm and respective options.

Stan provides several inference algorithms, parameterized pseudo-random variable generation,
and diagnostic tools. Each of these methods supports a variety of configuration options.

The methods and their configuration are abstracted into the `Method` enum, which provides
a uniform interface for use with `ArgTree`, which contains the rest of a user's configuration.

As a user, you will typically construct a given variant of `Method` using the respective
builder. When using a builder, note that one need only specify the variables which one
desires to be something other than the default value; default values are listed
in the documentation respective to each `struct` or `enum`.
*/
use crate::builder::Builder;
pub use crate::diagnose::*;
pub use crate::optimize::*;
pub use crate::sample::*;
use crate::translate::Translate;
pub use crate::variational::*;
use std::ffi::OsString;

/// Analysis method. Defaults to [`Method::Sample`].
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
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
        #[defaults_to = 1000]
        num_samples: i32,
        /// Number of warmup iterations
        /// Valid values: `0 <= warmup`.
        /// Defaults to `1000`.
        #[defaults_to = 1000]
        num_warmup: i32,
        /// Stream warmup samples to output?
        /// Defaults to `false`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        #[defaults_to = false]
        save_warmup: bool,
        /// Period between saved samples.
        /// Valid values: `0 < thin`.
        /// Defaults to `1`.
        #[defaults_to = 1]
        thin: i32,
        /// Warmup Adaptation
        adapt: SampleAdapt,
        /// Sampling algorithm. Defaults to [`SampleAlgorithm::Hmc`].
        algorithm: SampleAlgorithm,
        /// Number of chains.
        /// Valid values: `num_chains > 0`.
        /// Defaults to `1`.
        #[defaults_to = 1]
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
        #[defaults_to = false]
        jacobian: bool,
        /// Total number of iterations.
        /// Valid values: `0 < iter`.
        /// Defaults to `2000`.
        #[defaults_to = 2000]
        iter: i32,
        /// Stream optimization progress to output?
        /// Defaults to `false`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        #[defaults_to = false]
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
        #[defaults_to = 10000]
        iter: i32,
        /// Number of Monte Carlo draws for computing the gradient.
        /// Valid values: `0 < grad_samples`.
        /// Defaults to `1`.
        #[defaults_to = 1]
        grad_samples: i32,
        /// Number of Monte Carlo draws for estimate of ELBO.
        /// Valid values: `0 < elbo_samples`.
        /// Defaults to `100`.
        #[defaults_to = 100]
        elbo_samples: i32,
        /// Stepsize scaling parameter.
        /// Valid values: `0 < eta`.
        /// Defaults to `1.0`.
        #[defaults_to = 1.0]
        eta: f64,
        /// Eta Adaptation for Variational Inference.
        adapt: VariationalAdapt,
        /// Relative tolerance parameter for convergence.
        /// Valid values: `0 <= tol_rel_obj`.
        /// Defaults to `0.01`.
        #[defaults_to = 0.01]
        tol_rel_obj: f64,
        /// Number of iterations between ELBO evaluations.
        /// Valid values: `0 < eval_elbo`.
        /// Defaults to `100`.
        #[defaults_to = 100]
        eval_elbo: i32,
        /// Number of approximate posterior output draws to save.
        /// Valid values: `0 < output_samples`.
        /// Defaults to `1000`.
        #[defaults_to = 1000]
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
        #[defaults_to = ""]
        fitted_params: OsString,
    },
    /// Pathfinder algorithm. Use [`PathfinderBuilder`] for
    /// construction with defaults.
    #[non_exhaustive]
    Pathfinder {
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
        /// Defaults to `10000000.0`.
        #[defaults_to = "crate::consts::TOL_REL_GRAD"]
        tol_rel_grad: f64,
        /// Convergence tolerance on changes in parameter value.
        /// Valid values: `0 <= tol_param`.
        /// Defaults to `1e-08`
        #[defaults_to = "crate::consts::TOL_PARAM"]
        tol_param: f64,
        /// Amount of history to keep for L-BFGS.
        /// Valid values: `0 < history_size`.
        /// Defaults to `5`.
        #[defaults_to = "crate::consts::HISTORY_SIZE"]
        history_size: i32,
        /// Number of draws from PSIS sample.
        /// Valid values: `0 < num_psis_draws`.
        /// Defaults to `1000`.
        #[defaults_to = 1000]
        num_psis_draws: i32,
        /// Number of single pathfinders.
        /// Valid values: `0 < num_paths`.
        /// Defaults to `4`.
        #[defaults_to = 4]
        num_paths: i32,
        /// Output single-path pathfinder draws as CSV.
        /// Defaults to `false`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        #[defaults_to = false]
        save_single_paths: bool,
        /// Maximum number of LBFGS iterations.
        /// Valid values: `0 < max_lbfgs_iters`.
        /// Defaults to `1000`.
        #[defaults_to = 1000]
        max_lbfgs_iters: i32,
        /// Number of approximate posterior draws.
        /// Valid values: `0 < num_draws`.
        /// Defaults to `1000`.
        #[defaults_to = 1000]
        num_draws: i32,
        /// Number of Monte Carlo draws to evaluate ELBO.
        /// Valid values: `0 < num_elbo_draws`.
        /// Defaults to `25`.
        #[defaults_to = 25]
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
        #[defaults_to = ""]
        unconstrained_params: OsString,
        /// Input file (JSON or R dump) of parameter values on constrained scale.
        /// Valid values: Path to existing file.
        /// Defaults to `""`.
        #[defaults_to = ""]
        constrained_params: OsString,
        /// When true, include change-of-variables adjustment for
        /// constraining parameter transforms.
        /// Defaults to `true`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        #[defaults_to = true]
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
        #[defaults_to = ""]
        mode: OsString,
        /// When true, include change-of-variables adjustment for
        /// constraining parameter transforms.
        /// Defaults to `true`.
        ///
        /// At command line, this presents as `false` => 0, `true` => 1,
        /// with valid values 0 or 1.
        #[defaults_to = true]
        jacobian: bool,
        /// Number of draws from the laplace approximation.
        /// Valid values: `0 <= draws`.
        /// Defaults to `1000`.
        #[defaults_to = 1000]
        draws: i32,
    },
}

impl Default for Method {
    fn default() -> Self {
        SampleBuilder::new().build()
    }
}
// macro_rules! from_impl {
//     ($T:ident) => {
//         impl From<$T> for Method {
//             fn from(x: $T) -> Self {
//                 x.build()
//             }
//         }
//     };
// }
// from_impl!(SampleBuilder);
// from_impl!(OptimizeBuilder);
// from_impl!(VariationalBuilder);
// from_impl!(DiagnoseBuilder);
// from_impl!(GenerateQuantitiesBuilder);
// from_impl!(PathfinderBuilder);
// from_impl!(LogProbBuilder);
// from_impl!(LaplaceBuilder);

#[cfg(test)]
mod tests {
    use super::*;

    mod sample {
        use super::*;

        #[test]
        fn builder() {
            let x = SampleBuilder::new();
            let y = x.num_samples(2);
            let z = y.num_warmup(2);
            assert_eq!(z.num_samples, Some(2));
            assert_eq!(z.num_warmup, Some(2));

            let z = SampleBuilder::new()
                .num_samples(2)
                .num_warmup(2)
                .num_samples(10);
            assert_eq!(z.num_samples, Some(10));
            assert_eq!(z.num_warmup, Some(2));

            let x = SampleBuilder::new()
                .num_samples(2)
                .num_warmup(2)
                .save_warmup(true)
                .thin(5);
            assert_eq!(x.save_warmup, Some(true));
            assert_eq!(x.thin, Some(5));

            let x = SampleBuilder::new()
                .algorithm(SampleAlgorithm::default())
                .adapt(SampleAdapt::default());
            assert_eq!(x.adapt, Some(SampleAdapt::default()));
            assert_eq!(x.algorithm, Some(SampleAlgorithm::default()));

            let x = SampleBuilder::new()
                .num_samples(1)
                .num_warmup(2)
                .save_warmup(true)
                .thin(5)
                .num_chains(10);
            assert_eq!(x.num_chains, Some(10));

            // Default values
            let x = SampleBuilder::new().build();
            assert_eq!(
                x,
                Method::Sample {
                    num_samples: 1000,
                    num_warmup: 1000,
                    save_warmup: false,
                    thin: 1,
                    adapt: SampleAdapt::default(),
                    algorithm: SampleAlgorithm::default(),
                    num_chains: 1,
                }
            );
        }

        #[test]
        fn to_args() {
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
    }

    mod optimize {
        use super::*;

        #[test]
        fn builder() {
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
        fn to_args() {
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
    }

    mod variational {
        use super::*;

        #[test]
        fn builder() {
            let x = VariationalBuilder::new()
                .algorithm(VariationalAlgorithm::FullRank)
                .iter(1)
                .grad_samples(2)
                .elbo_samples(3)
                .eta(0.1)
                .adapt(VariationalAdapt::builder().engaged(false).iter(200).build())
                .tol_rel_obj(0.2)
                .eval_elbo(4)
                .output_samples(5)
                .build();
            assert_eq!(
                x,
                Method::Variational {
                    algorithm: VariationalAlgorithm::FullRank,
                    iter: 1,
                    grad_samples: 2,
                    elbo_samples: 3,
                    eta: 0.1,
                    adapt: VariationalAdapt::builder().engaged(false).iter(200).build(),
                    tol_rel_obj: 0.2,
                    eval_elbo: 4,
                    output_samples: 5
                }
            );

            let x = VariationalBuilder::new().build();
            assert_eq!(
                x,
                Method::Variational {
                    algorithm: VariationalAlgorithm::MeanField,
                    iter: 10000,
                    grad_samples: 1,
                    elbo_samples: 100,
                    eta: 1.0,
                    adapt: VariationalAdapt::default(),
                    tol_rel_obj: 0.01,
                    eval_elbo: 100,
                    output_samples: 1000,
                }
            );
        }

        #[test]
        fn to_args() {
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
    }

    mod diagnose {
        use super::*;

        #[test]
        fn builder() {
            let x = DiagnoseBuilder::new()
                .test(DiagnoseTest::Gradient {
                    epsilon: 1e-1,
                    error: 1e-1,
                })
                .build();
            assert_eq!(
                x,
                Method::Diagnose {
                    test: DiagnoseTest::Gradient {
                        epsilon: 1e-1,
                        error: 1e-1
                    }
                }
            );
        }

        #[test]
        fn to_args() {
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
    }

    mod generate_quantities {
        use super::*;

        #[test]
        fn builder() {
            let x = GenerateQuantitiesBuilder::new()
                .fitted_params("big.csv")
                .build();
            assert_eq!(
                x,
                Method::GenerateQuantities {
                    fitted_params: "big.csv".into()
                }
            );

            let x = GenerateQuantitiesBuilder::new().build();
            assert_eq!(
                x,
                Method::GenerateQuantities {
                    fitted_params: "".into()
                }
            );
        }

        #[test]
        fn to_args() {
            let x = GenerateQuantitiesBuilder::new().build();
            assert_eq!(
                x.to_args(),
                vec!["method=generate_quantities", "fitted_params="]
            );
        }
    }

    mod pathfinder {
        use super::*;

        #[test]
        fn builder() {
            let x = PathfinderBuilder::new()
                .init_alpha(0.1)
                .tol_obj(0.2)
                .tol_rel_obj(0.3)
                .tol_grad(0.4)
                .tol_rel_grad(0.5)
                .tol_param(0.6)
                .history_size(1)
                .num_psis_draws(2)
                .num_paths(3)
                .save_single_paths(true)
                .max_lbfgs_iters(4)
                .num_draws(5)
                .num_elbo_draws(6)
                .build();
            assert_eq!(
                x,
                Method::Pathfinder {
                    init_alpha: 0.1,
                    tol_obj: 0.2,
                    tol_rel_obj: 0.3,
                    tol_grad: 0.4,
                    tol_rel_grad: 0.5,
                    tol_param: 0.6,
                    history_size: 1,
                    num_psis_draws: 2,
                    num_paths: 3,
                    save_single_paths: true,
                    max_lbfgs_iters: 4,
                    num_draws: 5,
                    num_elbo_draws: 6,
                }
            );

            let x = PathfinderBuilder::new().build();
            assert_eq!(
                x,
                Method::Pathfinder {
                    init_alpha: 0.001,
                    tol_obj: 1e-12,
                    tol_rel_obj: 10_000.0,
                    tol_grad: 1e-8,
                    tol_rel_grad: 10_000_000.0,
                    tol_param: 1e-8,
                    history_size: 5,
                    num_psis_draws: 1000,
                    num_paths: 4,
                    save_single_paths: false,
                    max_lbfgs_iters: 1000,
                    num_draws: 1000,
                    num_elbo_draws: 25,
                }
            );
        }

        #[test]
        fn to_args() {
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
    }

    mod log_prob {
        use super::*;

        #[test]
        fn builder() {
            let x = LogProbBuilder::new()
                .unconstrained_params("unc.txt")
                .constrained_params("c.txt")
                .jacobian(false)
                .build();
            assert_eq!(
                x,
                Method::LogProb {
                    unconstrained_params: "unc.txt".into(),
                    constrained_params: "c.txt".into(),
                    jacobian: false
                }
            );
            let x = LogProbBuilder::new().build();
            assert_eq!(
                x,
                Method::LogProb {
                    unconstrained_params: "".into(),
                    constrained_params: "".into(),
                    jacobian: true
                }
            );
        }

        #[test]
        fn to_args() {
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
    }

    mod laplace {
        use super::*;

        #[test]
        fn builder() {
            let x = LaplaceBuilder::new()
                .mode("theta.json")
                .jacobian(false)
                .draws(10)
                .build();
            assert_eq!(
                x,
                Method::Laplace {
                    mode: "theta.json".into(),
                    jacobian: false,
                    draws: 10
                }
            );
            let x = LaplaceBuilder::new().build();
            assert_eq!(
                x,
                Method::Laplace {
                    mode: "".into(),
                    jacobian: true,
                    draws: 1000
                }
            );
        }

        #[test]
        fn to_args() {
            let x = LaplaceBuilder::new().build();
            assert_eq!(
                x.to_args(),
                vec!["method=laplace", "mode=", "jacobian=1", "draws=1000"]
            );
        }
    }
}
