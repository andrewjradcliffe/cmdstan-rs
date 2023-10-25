use crate::diagnose::*;
use crate::generate_quantities::*;
use crate::laplace::*;
use crate::logprob::*;
use crate::optimize::*;
use crate::sample::*;
use crate::variational::*;
use std::fmt::Write;

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    /// Bayesian inference using Markov Chain Monte Carlo
    Sample {
        /// Number of warmup iterations
        /// Valid values: 0 <= num_samples
        /// Defaults to 1000
        num_samples: i32,
        /// Number of warmup iterations
        /// Valid values: 0 <= warmup
        /// Defaults to 1000
        num_warmup: i32,
        /// Stream warmup samples to output?
        /// Valid values: [0, 1]
        /// Defaults to 0
        save_warmup: bool,
        /// Period between saved samples
        /// Valid values: 0 < thin
        /// Defaults to 1
        thin: i32,
        /// Warmup Adaptation
        adapt: SampleAdapt,
        /// Sampling algorithm
        algorithm: SampleAlgorithm,
        /// Number of chains
        /// Valid values: num_chains > 0
        /// Defaults to 1
        num_chains: i32,
    },
    /// Point estimation
    Optimize {
        algorithm: OptimizeAlgorithm,
        /// When true, include change-of-variables adjustment for constraining parameter transforms
        /// Valid values: [0, 1]
        /// Defaults to 0
        jacobian: bool,
        /// Total number of iterations
        /// Valid values: 0 < iter
        /// Defaults to 2000
        iter: i32,
        /// Stream optimization progress to output?
        /// Valid values: [0, 1]
        /// Defaults to 0
        save_iterations: bool,
    },
    /// Variational inference
    Variational {
        /// Variational inference algorithm
        /// Valid values: meanfield, fullrank
        /// Defaults to meanfield
        algorithm: VariationalAlgorithm,
        /// Maximum number of ADVI iterations.
        /// Valid values: 0 < iter
        /// Defaults to 10000
        iter: i32,
        /// Number of Monte Carlo draws for computing the gradient.
        /// Valid values: 0 < num_samples
        /// Defaults to 1
        grad_samples: i32,
        /// Number of Monte Carlo draws for estimate of ELBO.
        /// Valid values: 0 < num_samples
        /// Defaults to 100
        elbo_samples: i32,
        /// Stepsize scaling parameter.
        /// Valid values: 0 < eta
        /// Defaults to 1
        eta: f64,
        /// Eta Adaptation for Variational Inference
        /// Valid subarguments: engaged, iter
        adapt: VariationalAdapt,
        /// Relative tolerance parameter for convergence.
        /// Valid values: 0 <= tol
        /// Defaults to 0.01
        tol_rel_obj: f64,
        /// Number of iterations between ELBO evaluations
        /// Valid values: 0 < eval_elbo
        /// Defaults to 100
        eval_elbo: i32,
        /// Number of approximate posterior output draws to save.
        /// Valid values: 0 < output_samples
        /// Defaults to 1000
        output_samples: i32,
    },
    /// Model diagnostics
    Diagnose {
        /// Diagnostic test
        /// Valid values: gradient
        /// Defaults to gradient
        test: DiagnosticTest,
    },
    /// Generate quantities of interest
    GenerateQuantities {
        /// Input file of sample of fitted parameter values for model conditioned on data
        /// Valid values: Path to existing file
        /// Defaults to ""
        fitted_params: String,
    },
    /// Return the log density up to a constant and its gradients, given supplied parameters
    LogProb {
        /// Input file (JSON or R dump) of parameter values on unconstrained scale
        /// Valid values: Path to existing file
        /// Defaults to ""
        unconstrained_params: String,
        /// Input file (JSON or R dump) of parameter values on constrained scale
        /// Valid values: Path to existing file
        /// Defaults to ""
        constrained_params: String,
        /// When true, include change-of-variables adjustment for constraining parameter transforms
        /// Valid values: [0, 1]
        /// Defaults to 1
        jacobian: bool,
    },
    /// Sample from a Laplace approximation
    Laplace {
        /// A specification of a mode on the constrained scale for all model parameters, either in JSON or CSV format.
        /// Valid values: Path to existing file
        /// Defaults to ""
        mode: String,
        /// When true, include change-of-variables adjustment for constraining parameter transforms
        /// Valid values: [0, 1]
        /// Defaults to 1
        jacobian: bool,
        /// Number of draws from the laplace approximation
        /// Valid values: 0 <= draws
        /// Defaults to 1000
        draws: i32,
    },
}

impl Default for Method {
    fn default() -> Self {
        SampleBuilder::new().build()
    }
}
use Method::*;

impl From<SampleBuilder> for Method {
    fn from(x: SampleBuilder) -> Self {
        x.build()
    }
}
impl From<OptimizeBuilder> for Method {
    fn from(x: OptimizeBuilder) -> Self {
        x.build()
    }
}
impl From<VariationalBuilder> for Method {
    fn from(x: VariationalBuilder) -> Self {
        x.build()
    }
}
impl From<DiagnoseBuilder> for Method {
    fn from(x: DiagnoseBuilder) -> Self {
        x.build()
    }
}
impl From<GenerateQuantitiesBuilder> for Method {
    fn from(x: GenerateQuantitiesBuilder) -> Self {
        x.build()
    }
}
impl From<LogProbBuilder> for Method {
    fn from(x: LogProbBuilder) -> Self {
        x.build()
    }
}
impl From<LaplaceBuilder> for Method {
    fn from(x: LaplaceBuilder) -> Self {
        x.build()
    }
}

impl Method {
    pub fn command_fragment(&self) -> String {
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
                let mut s = String::from("method=sample");
                write!(&mut s, " num_samples={}", num_samples).unwrap();
                write!(&mut s, " num_warmup={}", num_warmup).unwrap();
                write!(&mut s, " save_warmup={}", *save_warmup as u8).unwrap();
                write!(&mut s, " thin={}", thin).unwrap();
                write!(&mut s, " {}", adapt.command_fragment()).unwrap();
                write!(&mut s, " {}", algorithm.command_fragment()).unwrap();
                write!(&mut s, " num_chains={}", num_chains).unwrap();
                s
            }
            Optimize {
                algorithm,
                jacobian,
                iter,
                save_iterations,
            } => {
                let mut s = String::from("method=optimize");
                write!(&mut s, " {}", algorithm.command_fragment()).unwrap();
                write!(&mut s, " jacobian={}", *jacobian as u8).unwrap();
                write!(&mut s, " iter={}", iter).unwrap();
                write!(&mut s, " save_iterations={}", *save_iterations as u8).unwrap();
                s
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
                let mut s = String::from("method=variational");
                write!(&mut s, " {}", algorithm.command_fragment()).unwrap();
                write!(&mut s, " iter={}", iter).unwrap();
                write!(&mut s, " grad_samples={}", grad_samples).unwrap();
                write!(&mut s, " elbo_samples={}", elbo_samples).unwrap();
                write!(&mut s, " eta={}", eta).unwrap();
                write!(&mut s, " {}", adapt.command_fragment()).unwrap();
                write!(&mut s, " tol_rel_obj={}", tol_rel_obj).unwrap();
                write!(&mut s, " eval_elbo={}", eval_elbo).unwrap();
                write!(&mut s, " output_samples={}", output_samples).unwrap();
                s
            }
            Diagnose { test } => {
                let mut s = String::from("method=diagnose");
                write!(&mut s, " {}", test.command_fragment()).unwrap();
                s
            }
            GenerateQuantities { fitted_params } => {
                format!("method=generate_quantities fitted_params={}", fitted_params)
            }
            LogProb {
                unconstrained_params,
                constrained_params,
                jacobian,
            } => {
                let mut s = String::from("method=log_prob");
                match unconstrained_params.as_ref() {
                    "" => (),
                    x => write!(&mut s, " unconstrained_params={}", x).unwrap(),
                };
                match constrained_params.as_ref() {
                    "" => (),
                    x => write!(&mut s, " constrained_params={}", x).unwrap(),
                };
                write!(&mut s, " jacobian={}", *jacobian as u8).unwrap();
                s
            }
            Laplace {
                mode,
                jacobian,
                draws,
            } => {
                let mut s = String::from("method=laplace");
                match mode.as_ref() {
                    "" => (),
                    x => write!(&mut s, " mode={}", x).unwrap(),
                };
                write!(&mut s, " jacobian={}", *jacobian as u8).unwrap();
                write!(&mut s, " draws={}", draws).unwrap();
                s
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_command_fragment() {
        let x = Method::default();
        assert_eq!(x.command_fragment(), "method=sample num_samples=1000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=10 metric=diag_e stepsize=1 stepsize_jitter=0 num_chains=1");
    }
}
