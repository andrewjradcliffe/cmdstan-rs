pub struct Model {
    stan_file: String,
    outdir: String,
}

#[derive(Debug)]
pub struct ArgumentTree {
    /// Analysis method
    /// Valid values: sample, optimize, variational, diagnose, generate_quantities, log_prob, laplace
    /// Defaults to sample
    method: Method,
    /// Unique process identifier
    /// Valid values: id >= 0
    /// Defaults to 1
    id: i32,
    /// Input data options
    data: Data,
    /// Initialization method: "x" initializes randomly between [-x, x], "0" initializes to 0, anything else identifies a file of values
    /// Valid values: All
    /// Defaults to "2"
    init: String, // Init,
    /// Random number configuration
    random: Random,
    /// File output options
    output: Output,
    /// Number of threads available to the program.
    /// Valid values: num_threads > 0 || num_threads == -1
    /// Defaults to 1 or the value of the STAN_NUM_THREADS environment variable if set.
    num_threads: i32,
}
impl Default for ArgumentTree {
    fn default() -> Self {
        Self {
            method: Method::default(),
            id: 1,
            data: Data::default(),
            init: String::from("2"),
            random: Random::default(),
            output: Output::default(),
            num_threads: 1,
        }
    }
}
#[derive(Debug)]
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
        adapt: SampleAdapt,
        algorithm: SampleAlgorithm,
        /// Number of chains
        /// Valid values: num_chains > 0
        /// Defaults to 1
        num_chains: i32,
    },
    /// Point estimation
    Optimize {
        algorithm: OptimizationAlgorithm,
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
        Method::Sample {
            num_samples: 1000,
            num_warmup: 1000,
            save_warmup: false,
            thin: 1,
            adapt: SampleAdapt::default(),
            algorithm: SampleAlgorithm::default(),
            num_chains: 1,
        }
    }
}

/// Warmup Adaptation
#[derive(Debug)]
pub struct SampleAdapt {
    /// Adaptation engaged?
    /// Valid values: [0, 1]
    /// Defaults to 1
    engaged: bool,
    /// Adaptation regularization scale
    /// Valid values: 0 < gamma
    /// Defaults to 0.05
    gamma: f64,
    /// Adaptation target acceptance statistic
    /// Valid values: 0 < delta < 1
    /// Defaults to 0.8
    delta: f64,
    /// Adaptation relaxation exponent
    /// Valid values: 0 < kappa
    /// Defaults to 0.75
    kappa: f64,
    /// Adaptation iteration offset
    /// Valid values: 0 < t0
    /// Defaults to 10
    t0: f64,
    /// Width of initial fast adaptation interval
    /// Valid values: All
    /// Defaults to 75
    init_buffer: u32,
    /// Width of final fast adaptation interval
    /// Valid values: All
    /// Defaults to 50
    term_buffer: u32,
    /// Initial width of slow adaptation interval
    /// Valid values: All
    /// Defaults to 25
    window: u32,
}
impl Default for SampleAdapt {
    fn default() -> Self {
        Self {
            engaged: true,
            gamma: 0.05,
            delta: 0.8,
            kappa: 0.75,
            t0: 10.0,
            init_buffer: 75,
            term_buffer: 50,
            window: 25,
        }
    }
}

/// Sampling algorithm
/// Valid values: hmc, fixed_param
/// Defaults to hmc
#[derive(Debug)]
pub enum SampleAlgorithm {
    /// Hamiltonian Monte Carlo
    Hmc {
        engine: Engine,
        metric: Metric,
        /// Input file with precomputed Euclidean metric
        /// Valid values: Path to existing file
        /// Defaults to ""
        metric_file: String,
        /// Step size for discrete evolution
        /// Valid values: 0 < stepsize
        /// Defaults to 1
        stepsize: f64,
        /// Uniformly random jitter of the stepsize, in percent
        /// Valid values: 0 <= stepsize_jitter <= 1
        /// Defaults to 0
        stepsize_jitter: f64,
    },
    /// Fixed Parameter Sampler
    FixedParam,
}

impl Default for SampleAlgorithm {
    fn default() -> Self {
        SampleAlgorithm::Hmc {
            engine: Engine::default(),
            metric: Metric::default(),
            metric_file: String::from(""),
            stepsize: 1.0,
            stepsize_jitter: 0.0,
        }
    }
}

/// Engine for Hamiltonian Monte Carlo
/// Valid values: static, nuts
/// Defaults to nuts
#[derive(Debug)]
pub enum Engine {
    /// Static integration time
    Static {
        /// Total integration time for Hamiltonian evolution
        /// Valid values: 0 < int_time
        /// Defaults to 2 * pi
        int_time: f64,
    },
    /// The No-U-Turn Sampler
    Nuts {
        /// Maximum tree depth
        /// Valid values: 0 < max_depth
        /// Defaults to 10
        max_depth: i32,
    },
}
impl Default for Engine {
    fn default() -> Self {
        Engine::Nuts { max_depth: 10 }
    }
}

/// Geometry of base manifold
/// Valid values: unit_e, diag_e, dense_e
/// Defaults to diag_e
#[derive(Debug, Default)]
pub enum Metric {
    /// Euclidean manifold with unit metric
    UnitE,
    /// Euclidean manifold with diag metric
    #[default]
    DiagE,
    /// Euclidean manifold with dense metric
    DenseE,
}

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

/// Variational inference algorithm
/// Valid values: meanfield, fullrank
/// Defaults to meanfield
#[derive(Debug, Default)]
pub enum VariationalAlgorithm {
    /// mean-field approximation
    #[default]
    MeanField,
    /// full-rank covariance
    FullRank,
}

/// Eta Adaptation for Variational Inference
/// Valid subarguments: engaged, iter
#[derive(Debug)]
pub struct VariationalAdapt {
    /// Boolean flag for eta adaptation.
    /// Valid values: [0, 1]
    /// Defaults to 1
    engaged: bool,
    /// Number of iterations for eta adaptation.
    /// Valid values: 0 < iter
    /// Defaults to 50
    iter: i32,
}
impl Default for VariationalAdapt {
    fn default() -> Self {
        Self {
            engaged: true,
            iter: 50,
        }
    }
}

#[derive(Debug)]
pub enum DiagnosticTest {
    /// Check model gradient against finite differences
    Gradient {
        /// Finite difference step size
        /// Valid values: 0 < epsilon
        /// Defaults to 1e-6
        epsilon: f64,
        /// Error threshold
        /// Valid values: 0 < error
        /// Defaults to 1e-6
        error: f64,
    },
}
impl Default for DiagnosticTest {
    fn default() -> Self {
        DiagnosticTest::Gradient {
            epsilon: 1e-6,
            error: 1e-6,
        }
    }
}

/// Input data options
#[derive(Debug)]
pub struct Data {
    /// Input data file
    /// Valid values: Path to existing file
    /// Defaults to ""
    file: String,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            file: String::from(""),
        }
    }
}

/// Random number configuration
#[derive(Debug)]
pub struct Random {
    /// Random number generator seed
    /// Valid values: non-negative integer < 4294967296  or -1 to generate seed from system time
    /// Defaults to -1
    seed: i64,
}
impl Default for Random {
    fn default() -> Self {
        Self { seed: -1 }
    }
}

/// File output options
#[derive(Debug)]
pub struct Output {
    /// Output file
    /// Valid values: Path to existing file
    /// Defaults to output.csv
    file: String,
    /// Auxiliary output file for diagnostic information
    /// Valid values: Path to existing file
    /// Defaults to ""
    diagnostic_file: String,
    /// Number of interations between screen updates
    /// Valid values: 0 <= refresh
    /// Defaults to 100
    refresh: i32,
    /// The number of significant figures used for the output CSV files.
    /// Valid values: 0 <= integer <= 18 or -1 to use the default number of significant figures
    /// Defaults to -1
    sig_figs: i32,
    /// File to store profiling information
    /// Valid values: Valid path and write acces to the folder
    /// Defaults to ""
    profile_file: String,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            file: String::from("output.csv"),
            diagnostic_file: String::from(""),
            refresh: 100,
            sig_figs: -1,
            profile_file: String::from(""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
