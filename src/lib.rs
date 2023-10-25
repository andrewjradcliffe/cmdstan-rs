use std::fmt::Write;

#[macro_use]
mod internal_macros;

pub mod diagnose;
pub mod generate_quantities;
pub mod laplace;
pub mod logprob;
pub mod method;
pub mod optimize;
pub mod sample;
pub mod variational;

pub use crate::method::*;

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
impl ArgumentTree {
    pub fn command_string(&self) -> String {
        let mut s = self.method.command_fragment();
        write!(&mut s, " id={}", self.id).unwrap();
        match self.data.command_fragment().as_ref() {
            "" => (),
            x => write!(&mut s, " {}", x).unwrap(),
        }
        write!(&mut s, " init={}", self.init).unwrap();
        write!(&mut s, " {}", self.random.command_fragment()).unwrap();
        write!(&mut s, " {}", self.output.command_fragment()).unwrap();
        write!(&mut s, " num_threads={}", self.num_threads).unwrap();
        s
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

impl Data {
    pub fn command_fragment(&self) -> String {
        let mut s = String::new();
        match self.file.as_ref() {
            "" => (),
            x => write!(&mut s, "data file={}", x).unwrap(),
        }
        s
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

impl Random {
    pub fn command_fragment(&self) -> String {
        format!("random seed={}", self.seed)
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

impl Output {
    pub fn command_fragment(&self) -> String {
        let mut s = format!("output file={}", self.file);
        match self.diagnostic_file.as_ref() {
            "" => (),
            x => write!(&mut s, " diagnostic_file={}", x).unwrap(),
        }
        write!(&mut s, " refresh={}", self.refresh).unwrap();
        write!(&mut s, " sig_figs={}", self.sig_figs).unwrap();
        match self.profile_file.as_ref() {
            "" => (),
            x => write!(&mut s, " profile_file={}", x).unwrap(),
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod argument_tree {
        use super::*;

        #[test]
        fn command_string() {
            let x = ArgumentTree::default();
            assert_eq!(x.command_string(), "method=sample num_samples=1000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=10 metric=diag_e stepsize=1 stepsize_jitter=0 num_chains=1 id=1 init=2 random seed=-1 output file=output.csv refresh=100 sig_figs=-1 num_threads=1");
        }
    }

    #[cfg(test)]
    mod data {
        use super::*;

        #[test]
        fn command_fragment() {
            let mut x = Data::default();
            assert_eq!(x.command_fragment(), "");

            x.file.push_str("bernoulli.data.json");
            assert_eq!(x.command_fragment(), "data file=bernoulli.data.json");
        }
    }

    #[cfg(test)]
    mod random {
        use super::*;

        #[test]
        fn command_fragment() {
            let x = Random::default();
            assert_eq!(x.command_fragment(), "random seed=-1");
        }
    }

    #[cfg(test)]
    mod output {
        use super::*;
        #[test]
        fn command_fragment() {
            let mut x = Output::default();
            assert_eq!(
                x.command_fragment(),
                "output file=output.csv refresh=100 sig_figs=-1"
            );

            x.diagnostic_file.push_str("my_file.txt");
            assert_eq!(
                x.command_fragment(),
                "output file=output.csv diagnostic_file=my_file.txt refresh=100 sig_figs=-1"
            );

            x.profile_file.push_str("my_other_file.txt");
            assert_eq!(
                x.command_fragment(),
                "output file=output.csv diagnostic_file=my_file.txt refresh=100 sig_figs=-1 profile_file=my_other_file.txt"
            );

            x.diagnostic_file.clear();
            assert_eq!(
                x.command_fragment(),
                "output file=output.csv refresh=100 sig_figs=-1 profile_file=my_other_file.txt"
            );
        }
    }
}
