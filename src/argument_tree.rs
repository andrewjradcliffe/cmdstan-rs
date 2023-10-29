use crate::method::*;
use std::env;
use std::fmt::Write;

#[derive(Debug, PartialEq, Clone)]
pub struct ArgumentTree {
    /// Analysis method. Defaults to `Sample`.
    pub method: Method,
    /// Unique process identifier
    /// Valid values: id >= 0
    /// Defaults to 1
    pub id: i32,
    /// Input data options
    pub data: Data,
    /// Initialization method: "x" initializes randomly between [-x, x], "0" initializes to 0, anything else identifies a file of values
    /// Valid values: All
    /// Defaults to "2"
    pub init: String,
    /// Random number configuration
    pub random: Random,
    /// File output options
    pub output: Output,
    /// Number of threads available to the program.
    /// Valid values: num_threads > 0 || num_threads == -1
    /// Defaults to 1 or the value of the STAN_NUM_THREADS environment variable if set.
    pub num_threads: i32,
}
impl Default for ArgumentTree {
    fn default() -> Self {
        ArgumentTreeBuilder::new().build()
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
    /// Return a builder with all options unspecified.
    pub fn builder() -> ArgumentTreeBuilder {
        ArgumentTreeBuilder::new()
    }

    pub fn output_files(&self) -> Vec<String> {
        let mut files: Vec<String> = Vec::new();
        let output_file = &self.output.file;
        let prefix = match output_file.rsplit_once(".csv") {
            Some((prefix, _)) => prefix,
            None => output_file,
        };
        match &self.method {
            Method::Sample { num_chains, .. } => {
                if *num_chains != 1 {
                    let id = self.id.clone();
                    (id..id + num_chains).for_each(|id| {
                        files.push(format!("{prefix}_{id}.csv"));
                    });
                } else {
                    files.push(format!("{prefix}.csv"));
                }
            }
            _ => {
                files.push(format!("{prefix}.csv"));
            }
        }
        files
    }
}

/// Options builder for `ArgumentTree`.
/// For any option left unspecified, the default value indicated
/// on `ArgumentTree` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct ArgumentTreeBuilder {
    method: Option<Method>,
    id: Option<i32>,
    data: Option<Data>,
    init: Option<String>,
    random: Option<Random>,
    output: Option<Output>,
    num_threads: Option<i32>,
}
impl ArgumentTreeBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            method: None,
            id: None,
            data: None,
            init: None,
            random: None,
            output: None,
            num_threads: None,
        }
    }
    insert_field!(method, Method);
    // pub fn method<T: Into<Method>>(self, method: T) -> Self {
    //     let mut me = self;
    //     let _ = me.method.insert(method.into());
    // }
    insert_field!(id, i32);
    insert_field!(data, Data);
    insert_field!(init, String);
    // insert_string_field!(init);
    insert_field!(random, Random);
    insert_field!(output, Output);
    insert_field!(num_threads, i32);
    /// Build the `ArgumentTree` instance.
    pub fn build(self) -> ArgumentTree {
        let method = self.method.unwrap_or_default();
        let id = self.id.unwrap_or(1);
        let data = self.data.unwrap_or_default();
        let init = self.init.unwrap_or_else(|| "2".to_string());
        let random = self.random.unwrap_or_default();
        let output = self.output.unwrap_or_default();
        let num_threads = self.num_threads.unwrap_or_else(|| {
            env::var("STAN_NUM_THREADS").map_or(1, |s| s.parse::<i32>().unwrap_or(1))
        });
        ArgumentTree {
            method,
            id,
            data,
            init,
            random,
            output,
            num_threads,
        }
    }
}

/// Input data options
#[derive(Debug, PartialEq, Clone)]
pub struct Data {
    /// Input data file
    /// Valid values: Path to existing file
    /// Defaults to ""
    pub file: String,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            file: "".to_string(),
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
#[derive(Debug, PartialEq, Clone)]
pub struct Random {
    /// Random number generator seed
    /// Valid values: non-negative integer < 4294967296  or -1 to generate seed from system time
    /// Defaults to -1
    pub seed: i64,
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
#[derive(Debug, PartialEq, Clone)]
pub struct Output {
    /// Output file
    /// Valid values: Path to existing file
    /// Defaults to output.csv
    pub file: String,
    /// Auxiliary output file for diagnostic information
    /// Valid values: Path to existing file
    /// Defaults to ""
    pub diagnostic_file: String,
    /// Number of interations between screen updates
    /// Valid values: 0 <= refresh
    /// Defaults to 100
    pub refresh: i32,
    /// The number of significant figures used for the output CSV files.
    /// Valid values: 0 <= integer <= 18 or -1 to use the default number of significant figures
    /// Defaults to -1
    pub sig_figs: i32,
    /// File to store profiling information
    /// Valid values: Valid path and write acces to the folder
    /// Defaults to ""
    pub profile_file: String,
}

impl Default for Output {
    fn default() -> Self {
        OutputBuilder::new().build()
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
    /// Return a builder with all options unspecified.
    pub fn builder() -> OutputBuilder {
        OutputBuilder::new()
    }
}

/// Options builder for `Output`.
/// For any option left unspecified, the default value indicated
/// on `Output` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct OutputBuilder {
    file: Option<String>,
    diagnostic_file: Option<String>,
    refresh: Option<i32>,
    sig_figs: Option<i32>,
    profile_file: Option<String>,
}

impl OutputBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            file: None,
            diagnostic_file: None,
            refresh: None,
            sig_figs: None,
            profile_file: None,
        }
    }
    insert_field!(file, String);
    insert_field!(diagnostic_file, String);
    // insert_string_field!(diagnostic_file);
    insert_field!(refresh, i32);
    insert_field!(sig_figs, i32);
    insert_field!(profile_file, String);
    /// Build the `Output` instance.
    pub fn build(self) -> Output {
        let file = self.file.unwrap_or_else(|| "output.csv".to_string());
        let diagnostic_file = self.diagnostic_file.unwrap_or_else(|| "".to_string());
        let refresh = self.refresh.unwrap_or(100);
        let sig_figs = self.sig_figs.unwrap_or(-1);
        let profile_file = self.profile_file.unwrap_or_else(|| "".to_string());
        Output {
            file,
            diagnostic_file,
            refresh,
            sig_figs,
            profile_file,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod argument_tree {
        use super::*;
        use crate::sample::*;

        #[test]
        fn builder() {
            let method = SampleBuilder::new().num_chains(10_000).build();
            let id = 2;
            let data = Data {
                file: "bernoulli.json".to_string(),
            };
            let init = "5".to_string();
            let random = Random { seed: 12345 };
            let output = Output {
                file: "hello.csv".to_string(),
                diagnostic_file: "world.txt".to_string(),
                refresh: 1,
                sig_figs: 18,
                profile_file: "foo.txt".to_string(),
            };
            let num_threads = 48;
            let x = ArgumentTree::builder()
                .method(method.clone())
                .id(id)
                .data(data.clone())
                .init(init.clone())
                .random(random.clone())
                .output(output.clone())
                .num_threads(num_threads)
                .build();
            assert_eq!(
                x,
                ArgumentTree {
                    method,
                    id,
                    data,
                    init,
                    random,
                    output,
                    num_threads,
                }
            );
        }

        #[test]
        fn default() {
            let method = SampleBuilder::new().build();
            let id = 1;
            let data = Data {
                file: "".to_string(),
            };
            let init = "2".to_string();
            let random = Random { seed: -1 };
            let output = Output {
                file: "output.csv".to_string(),
                diagnostic_file: "".to_string(),
                refresh: 100,
                sig_figs: -1,
                profile_file: "".to_string(),
            };
            let num_threads = 1;
            assert_eq!(
                ArgumentTree::default(),
                ArgumentTree {
                    method,
                    id,
                    data,
                    init,
                    random,
                    output,
                    num_threads,
                }
            );
        }

        #[test]
        fn command_string() {
            let x = ArgumentTree::default();
            assert_eq!(x.command_string(), "method=sample num_samples=1000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=10 metric=diag_e stepsize=1 stepsize_jitter=0 num_chains=1 id=1 init=2 random seed=-1 output file=output.csv refresh=100 sig_figs=-1 num_threads=1");

            let method = SampleBuilder::new()
                .num_chains(10)
                .num_samples(10_000)
                .algorithm(
                    HmcBuilder::new()
                        .engine(NutsBuilder::new().max_depth(100).build())
                        .build(),
                )
                .build();
            let id = 2;
            let data = Data {
                file: "bernoulli.json".to_string(),
            };
            let init = "5".to_string();
            let random = Random { seed: 12345 };
            let output = Output {
                file: "hello.csv".to_string(),
                diagnostic_file: "world.txt".to_string(),
                refresh: 1,
                sig_figs: 18,
                profile_file: "foo.txt".to_string(),
            };
            let num_threads = 48;
            let x = ArgumentTree {
                method,
                id,
                data,
                init,
                random,
                output,
                num_threads,
            };
            assert_eq!(x.command_string(), "method=sample num_samples=10000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=100 metric=diag_e stepsize=1 stepsize_jitter=0 num_chains=10 id=2 data file=bernoulli.json init=5 random seed=12345 output file=hello.csv diagnostic_file=world.txt refresh=1 sig_figs=18 profile_file=foo.txt num_threads=48");

            let method = SampleBuilder::new()
                .num_chains(10)
                .num_samples(10_000)
                .algorithm(
                    HmcBuilder::new()
                        .engine(StaticBuilder::new().int_time(2.5).build())
                        .build(),
                )
                .build();
            let id = 2;
            let data = Data {
                file: "bernoulli.json".to_string(),
            };
            let init = "5".to_string();
            let random = Random { seed: 12345 };
            let output = Output {
                file: "hello.csv".to_string(),
                diagnostic_file: "world.txt".to_string(),
                refresh: 1,
                sig_figs: 18,
                profile_file: "foo.txt".to_string(),
            };
            let num_threads = 48;
            let x = ArgumentTree {
                method,
                id,
                data,
                init,
                random,
                output,
                num_threads,
            };
            assert_eq!(x.command_string(), "method=sample num_samples=10000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=static int_time=2.5 metric=diag_e stepsize=1 stepsize_jitter=0 num_chains=10 id=2 data file=bernoulli.json init=5 random seed=12345 output file=hello.csv diagnostic_file=world.txt refresh=1 sig_figs=18 profile_file=foo.txt num_threads=48");
        }
    }

    #[cfg(test)]
    mod data {
        use super::*;

        #[test]
        fn default() {
            let x = Data::default();
            assert_eq!(x.file, "");
        }

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
        fn default() {
            let x = Random::default();
            assert_eq!(x.seed, -1_i64);
        }

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
        fn builder() {
            let x = Output::builder()
                .file("hello.csv".to_string())
                .diagnostic_file("world.txt".to_string())
                .refresh(1)
                .sig_figs(18)
                .profile_file("foo.txt".to_string())
                .build();
            assert_eq!(
                x,
                Output {
                    file: "hello.csv".to_string(),
                    diagnostic_file: "world.txt".to_string(),
                    refresh: 1,
                    sig_figs: 18,
                    profile_file: "foo.txt".to_string(),
                }
            );
        }

        #[test]
        fn default() {
            let x = Output::default();
            assert_eq!(
                x,
                Output {
                    file: "output.csv".to_string(),
                    diagnostic_file: "".to_string(),
                    refresh: 100,
                    sig_figs: -1,
                    profile_file: "".to_string(),
                }
            );
        }

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
