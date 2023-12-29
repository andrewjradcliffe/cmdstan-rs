use crate::method::*;
use crate::builder::Builder;
use crate::translate::Translate;
use std::ffi::{OsStr, OsString};

const NEG1_I32: i32 = -1;
const NEG1_I64: i64 = -1;
const OUTPUT_FILE: &str = "output.csv";
const PROFILE_FILE: &str = "profile.csv";

#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
// Lack of `declare` is intentional.
pub struct ArgTree {
    /// Analysis method. Defaults to [`Method::Sample`].
    pub method: Method,
    /// Unique process identifier.
    /// Valid values: `id >= 0`.
    /// Defaults to `1`.
    #[defaults_to = 1]
    pub id: i32,
    /// Input data options
    pub data: Data,
    /// Initialization method: `"x"` initializes randomly between [-x,
    /// x], `"0"` initializes to `0`, anything else identifies a file of
    /// values.
    /// Valid values: All.
    /// Defaults to `"2"`.
    #[defaults_to = "2"]
    pub init: OsString,
    /// Random number configuration
    pub random: Random,
    /// File output options
    pub output: Output,
    /// Number of threads available to the program.
    /// Valid values: `num_threads > 0 || num_threads == -1`.
    /// Defaults to `1` or the value of the `STAN_NUM_THREADS` environment variable if set.
    #[defaults_to = 1]
    pub num_threads: i32,
}

/// Match the behavior of CmdStan path handling, which
/// includes substitution of a `"csv"` suffix if no `'.'`
/// is present in the input.
fn rsplit_file_at_dot<'a>(file: &'a OsStr) -> (&'a OsStr, &'a OsStr) {
    let bytes = file.as_encoded_bytes();
    let mut iter = bytes.rsplitn(2, |b| *b == b'.');

    let (prefix, suffix) = match (iter.next(), iter.next()) {
        (Some(suffix), Some(prefix)) => {
            // SAFETY:
            // - each fragment only contains content that originated
            //   from `OsStr::as_encoded_bytes`.
            // - split with ASCII period, which is a non-empty UTF-8
            //   substring.
            // Thus, the invariants are maintained.
            unsafe {
                (
                    OsStr::from_encoded_bytes_unchecked(prefix),
                    OsStr::from_encoded_bytes_unchecked(suffix),
                )
            }
        }
        _ => (file, "csv".as_ref()),
    };
    (prefix, suffix)
}


/** File-handling utilities. */
impl ArgTree {
    fn files<F>(&self, f: F) -> Vec<OsString>
    where
        F: Fn(&ArgTree) -> &OsStr,
    {
        let mut files: Vec<OsString> = Vec::new();
        let file = f(self);
        let (prefix, suffix) = rsplit_file_at_dot(file);
        match &self.method {
            Method::Sample { num_chains, .. } if *num_chains != 1 => {
                let id = self.id;
                (id..id + num_chains).for_each(|id| {
                    let mut s = prefix.to_os_string();
                    s.push(format!("_{id}."));
                    s.push(suffix);
                    files.push(s);
                });
            }
            _ => {
                let mut s = prefix.to_os_string();
                s.push(".");
                s.push(suffix);
                files.push(s);
            }
        }
        files
    }

    /// Return the output file path(s), as implied by the configuration of `self`.
    /// Typically, these will not be literal files on the filesystem.
    pub fn output_files(&self) -> Vec<OsString> {
        self.files(|tree| &tree.output.file)
    }
    /// Return the diagnostic file path(s), as implied by the configuration of `self`.
    /// Typically, these will not be literal files on the filesystem.
    pub fn diagnostic_files(&self) -> Vec<OsString> {
        if self.output.diagnostic_file.is_empty() {
            Vec::new()
        } else {
            self.files(|tree| &tree.output.diagnostic_file)
        }
    }
    /// Return the profile file path(s), as implied by the configuration of `self`.
    /// Typically, these will not be literal files on the filesystem.
    pub fn profile_files(&self) -> Vec<OsString> {
        vec![self.output.profile_file.clone()]
    }
    /// Return the single-path pathfinder file path(s), if
    /// appropriate, as implied by the configuration of `self`.
    /// Typically, these will not be literal files on the filesystem.
    pub fn single_path_pathfinder_files(&self) -> Option<Vec<OsString>> {
        match &self.method {
            Method::Pathfinder {
                save_single_paths,
                num_paths,
                ..
            } => {
                let mut files: Vec<OsString> = Vec::new();
                if *save_single_paths {
                    let file: &OsStr = self.output.file.as_ref();
                    // Note that at present, it is easy to confuse `CmdStan` with
                    // too many '.' interspersed in self.output.file.
                    // Thus, this may not necessarily reproduce the files
                    // particularly well.
                    let (prefix, _) = rsplit_file_at_dot(file);
                    if *num_paths != 1 {
                        let id = self.id;
                        (id..id + num_paths).for_each(|id| {
                            let mut s1 = prefix.to_os_string();
                            s1.push(format!("_path_{id}."));
                            let mut s2 = s1.clone();
                            s1.push("csv");
                            s2.push("json");
                            files.push(s1);
                            files.push(s2);
                        });
                    } else {
                        let mut s1 = prefix.to_os_string();
                        let mut s2 = s1.clone();
                        s1.push(".csv");
                        s2.push(".json");
                        files.push(s1);
                        files.push(s2);
                    }
                }
                Some(files)
            }
            _ => None,
        }
    }
}

/// Input data options
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "data"]
pub struct Data {
    /// Input data file.
    /// Valid values: Path to existing file.
    /// Defaults to `""`.
    #[defaults_to = ""]
    pub file: OsString,
}

/// Random number configuration
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "random"]
pub struct Random {
    /// Random number generator seed.
    /// Valid values: non-negative integer < `4294967296` or `-1` to
    /// generate seed from system time.
    /// Defaults to `-1`.
    #[defaults_to = "NEG1_I64"]
    pub seed: i64,
}

/// File output options
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "output"]
pub struct Output {
    /// Output file.
    /// Valid values: Path to existing file.
    /// Defaults to `"output.csv"`.
    #[defaults_to = "OUTPUT_FILE"]
    pub file: OsString,
    /// Auxiliary output file for diagnostic information.
    /// Valid values: Path to existing file.
    /// Defaults to `""`.
    #[defaults_to = ""]
    pub diagnostic_file: OsString,
    /// Number of interations between screen updates.
    /// Valid values: `0 <= refresh`.
    /// Defaults to `100`.
    #[defaults_to = 100]
    pub refresh: i32,
    /// The number of significant figures used for the output CSV
    /// files.
    /// Valid values: `0 <= sig_figs <= 18` or `-1` to use the
    /// default number of significant figures.
    /// Defaults to` -1`.
    #[defaults_to = "NEG1_I32"]
    pub sig_figs: i32,
    /// File to store profiling information.
    /// Valid values: Valid path and write access to the folder.
    /// Defaults to `"profile.csv"`.
    #[defaults_to = "PROFILE_FILE"]
    pub profile_file: OsString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod argtree {
        use super::*;
        use crate::sample::*;

        #[test]
        fn builder() {
            let method = SampleBuilder::new().num_chains(10_000).build();
            let id = 2;
            let data = Data {
                file: "bernoulli.json".into(),
            };
            let init: OsString = "5".into();
            let random = Random { seed: 12345 };
            let output = Output {
                file: "hello.csv".into(),
                diagnostic_file: "world.txt".into(),
                refresh: 1,
                sig_figs: 18,
                profile_file: "foo.txt".into(),
            };
            let num_threads = 48;
            let x = ArgTree::builder()
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
                ArgTree {
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
            let data = Data { file: "".into() };
            let init: OsString = "2".into();
            let random = Random { seed: -1 };
            let output = Output {
                file: "output.csv".into(),
                diagnostic_file: "".into(),
                refresh: 100,
                sig_figs: -1,
                profile_file: "profile.csv".into(),
            };
            let num_threads = 1;
            assert_eq!(
                ArgTree::default(),
                ArgTree {
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
        fn to_stmt() {
            let x = ArgTree::default();
            assert_eq!(x.to_stmt(), "method=sample num_samples=1000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=10 metric=diag_e metric_file= stepsize=1 stepsize_jitter=0 num_chains=1 id=1 data file= init=2 random seed=-1 output file=output.csv diagnostic_file= refresh=100 sig_figs=-1 profile_file=profile.csv num_threads=1");

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
                file: "bernoulli.json".into(),
            };
            let init: OsString = "5".into();
            let random = Random { seed: 12345 };
            let output = Output {
                file: "hello.csv".into(),
                diagnostic_file: "world.txt".into(),
                refresh: 1,
                sig_figs: 18,
                profile_file: "foo.txt".into(),
            };
            let num_threads = 48;
            let x = ArgTree {
                method,
                id,
                data,
                init,
                random,
                output,
                num_threads,
            };
            assert_eq!(x.to_stmt(), "method=sample num_samples=10000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=100 metric=diag_e metric_file= stepsize=1 stepsize_jitter=0 num_chains=10 id=2 data file=bernoulli.json init=5 random seed=12345 output file=hello.csv diagnostic_file=world.txt refresh=1 sig_figs=18 profile_file=foo.txt num_threads=48");

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
                file: "bernoulli.json".into(),
            };
            let init: OsString = "5".into();
            let random = Random { seed: 12345 };
            let output = Output {
                file: "hello.csv".into(),
                diagnostic_file: "world.txt".into(),
                refresh: 1,
                sig_figs: 18,
                profile_file: "foo.txt".into(),
            };
            let num_threads = 48;
            let x = ArgTree {
                method,
                id,
                data,
                init,
                random,
                output,
                num_threads,
            };
            assert_eq!(x.to_stmt(), "method=sample num_samples=10000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=static int_time=2.5 metric=diag_e metric_file= stepsize=1 stepsize_jitter=0 num_chains=10 id=2 data file=bernoulli.json init=5 random seed=12345 output file=hello.csv diagnostic_file=world.txt refresh=1 sig_figs=18 profile_file=foo.txt num_threads=48");
        }

        #[test]
        fn files() {
            let b = ArgTree::builder()
                .method(SampleBuilder::new().num_chains(3))
                .id(2);
            let x = b
                .clone()
                .output(Output::builder().file("post").diagnostic_file("checks"))
                .build();
            assert_eq!(
                x.output_files(),
                vec!["post_2.csv", "post_3.csv", "post_4.csv"]
            );
            assert_eq!(
                x.diagnostic_files(),
                vec!["checks_2.csv", "checks_3.csv", "checks_4.csv"]
            );

            let x = b
                .clone()
                .output(
                    Output::builder()
                        .file("world.hello")
                        .diagnostic_file("goodbye.world"),
                )
                .build();
            assert_eq!(
                x.output_files(),
                vec!["world_2.hello", "world_3.hello", "world_4.hello"]
            );
            assert_eq!(
                x.diagnostic_files(),
                vec!["goodbye_2.world", "goodbye_3.world", "goodbye_4.world"]
            );

            let x = b
                .clone()
                .output(Output::builder().file("a.b.c").diagnostic_file("a...,"))
                .build();
            assert_eq!(x.output_files(), vec!["a.b_2.c", "a.b_3.c", "a.b_4.c"]);
            assert_eq!(x.diagnostic_files(), vec!["a.._2.,", "a.._3.,", "a.._4.,"]);

            let x = b
                .clone()
                .output(Output::builder().file("...xyz").diagnostic_file("abc..."))
                .build();
            assert_eq!(x.output_files(), vec![".._2.xyz", ".._3.xyz", ".._4.xyz"]);
            assert_eq!(
                x.diagnostic_files(),
                vec!["abc.._2.", "abc.._3.", "abc.._4."]
            );

            let x = b.clone().output(Output::builder().file("foo.")).build();
            assert_eq!(x.output_files(), vec!["foo_2.", "foo_3.", "foo_4."]);
            let x = b.clone().output(Output::builder().file("foo..")).build();
            assert_eq!(x.output_files(), vec!["foo._2.", "foo._3.", "foo._4."]);

            let x = b
                .clone()
                .output(Output::builder().file(",,").diagnostic_file(","))
                .build();
            assert_eq!(x.output_files(), vec![",,_2.csv", ",,_3.csv", ",,_4.csv"]);
            assert_eq!(x.diagnostic_files(), vec![",_2.csv", ",_3.csv", ",_4.csv"]);

            let x = b
                .clone()
                .output(Output::builder().file(".xyz").diagnostic_file(".txt"))
                .build();
            assert_eq!(x.output_files(), vec!["_2.xyz", "_3.xyz", "_4.xyz"]);
            assert_eq!(x.diagnostic_files(), vec!["_2.txt", "_3.txt", "_4.txt"]);

            let x = b.clone().output(Output::builder().file(".")).build();
            assert_eq!(x.output_files(), vec!["_2.", "_3.", "_4."]);
            let x = b.clone().output(Output::builder().file("..")).build();
            assert_eq!(x.output_files(), vec!["._2.", "._3.", "._4."]);
            let x = b.clone().output(Output::builder().file("...")).build();
            assert_eq!(x.output_files(), vec![".._2.", ".._3.", ".._4."]);

            let x = b.clone().output(Output::builder().file("foo/.bar")).build();
            assert_eq!(
                x.output_files(),
                vec!["foo/_2.bar", "foo/_3.bar", "foo/_4.bar"]
            );
            let x = b.clone().output(Output::builder().file("foo/bar/")).build();
            assert_eq!(
                x.output_files(),
                vec!["foo/bar/_2.csv", "foo/bar/_3.csv", "foo/bar/_4.csv"]
            );
            let x = b
                .clone()
                .output(Output::builder().file("foo/bar/."))
                .build();
            assert_eq!(
                x.output_files(),
                vec!["foo/bar/_2.", "foo/bar/_3.", "foo/bar/_4."]
            );
            let x = b
                .clone()
                .output(Output::builder().file("foo/bar/.."))
                .build();
            assert_eq!(
                x.output_files(),
                vec!["foo/bar/._2.", "foo/bar/._3.", "foo/bar/._4."]
            );
            let x = b
                .clone()
                .output(Output::builder().file("foo/bar/..."))
                .build();
            assert_eq!(
                x.output_files(),
                vec!["foo/bar/.._2.", "foo/bar/.._3.", "foo/bar/.._4."]
            );

            let x = b
                .clone()
                .output(Output::builder().file("foo/bar.baz."))
                .build();
            assert_eq!(
                x.output_files(),
                vec!["foo/bar.baz_2.", "foo/bar.baz_3.", "foo/bar.baz_4."]
            );

            let x = b
                .clone()
                .output(Output::builder().file("foo/bar/baz."))
                .build();
            assert_eq!(
                x.output_files(),
                vec!["foo/bar/baz_2.", "foo/bar/baz_3.", "foo/bar/baz_4."]
            );
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
        fn to_args() {
            let mut x = Data::default();
            assert_eq!(x.to_args(), vec!["data", "file="]);

            x.file.push("bernoulli.data.json");
            assert_eq!(x.to_args(), vec!["data", "file=bernoulli.data.json"]);
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
        fn to_args() {
            let x = Random::default();
            assert_eq!(x.to_args(), vec!["random", "seed=-1"]);
        }
    }

    #[cfg(test)]
    mod output {
        use super::*;

        #[test]
        fn builder() {
            let x = Output::builder()
                .file("hello.csv")
                .diagnostic_file("world.txt")
                .refresh(1)
                .sig_figs(18)
                .profile_file("foo.txt")
                .build();
            assert_eq!(
                x,
                Output {
                    file: "hello.csv".into(),
                    diagnostic_file: "world.txt".into(),
                    refresh: 1,
                    sig_figs: 18,
                    profile_file: "foo.txt".into(),
                }
            );
        }

        #[test]
        fn default() {
            let x = Output::default();
            assert_eq!(
                x,
                Output {
                    file: "output.csv".into(),
                    diagnostic_file: "".into(),
                    refresh: 100,
                    sig_figs: -1,
                    profile_file: "profile.csv".into(),
                }
            );
        }

        #[test]
        fn to_args() {
            let mut x = Output::default();
            assert_eq!(
                x.to_args(),
                vec![
                    "output",
                    "file=output.csv",
                    "diagnostic_file=",
                    "refresh=100",
                    "sig_figs=-1",
                    "profile_file=profile.csv"
                ]
            );

            x.diagnostic_file.push("my_file.txt");
            assert_eq!(
                x.to_args(),
                vec![
                    "output",
                    "file=output.csv",
                    "diagnostic_file=my_file.txt",
                    "refresh=100",
                    "sig_figs=-1",
                    "profile_file=profile.csv",
                ]
            );

            x.profile_file = "my_other_file.txt".into();
            assert_eq!(
                x.to_args(),
                vec![
                    "output",
                    "file=output.csv",
                    "diagnostic_file=my_file.txt",
                    "refresh=100",
                    "sig_figs=-1",
                    "profile_file=my_other_file.txt"
                ]
            );

            x.diagnostic_file.clear();
            assert_eq!(
                x.to_args(),
                vec![
                    "output",
                    "file=output.csv",
                    "diagnostic_file=",
                    "refresh=100",
                    "sig_figs=-1",
                    "profile_file=my_other_file.txt"
                ]
            );
        }
    }
}
