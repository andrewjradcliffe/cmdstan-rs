use crate::argument_tree::ArgumentTree;
use std::fmt::Write;
use std::process::{self, Command};
use std::{ffi, fs, io, path::Path, path::PathBuf, str};
use thiserror::Error;

/// Structure to direct compilation and execution of a Stan model.
/// Computation of diagnostics and summaries for said model are
/// facilitated through the same interface.
#[derive(Debug, PartialEq, Clone)]
pub struct Control {
    cmdstan: PathBuf,
    model: PathBuf,
}

#[derive(Error, Debug)]
pub enum CompilationError {
    #[error("Something happened to the process: {0:?}")]
    ProcessError(io::Error),
    #[error("Make problem: {0}")]
    MakeError(String),
    #[error("Compilation problem: {0:?}")]
    StanCompilerError(process::Output),
    #[error("Pre-existing binary artifacts cannot be removed: {0:?}")]
    DirtyWorkspaceError(io::Error),
}
use CompilationError::*;

#[cfg(unix)]
static MAKE: &'static str = "make";
#[cfg(windows)]
static MAKE: &'static str = "mingw32-make";

impl Control {
    /// Construct a new instance from a path (`cmdstan`) to a
    /// [`CmdStan`](https://mc-stan.org/docs/cmdstan-guide/cmdstan-installation.html)
    /// installation and a path (`model`) to a Stan program.
    pub fn new(cmdstan: &Path, model: &Path) -> Self {
        Self {
            cmdstan: PathBuf::from(cmdstan),
            model: PathBuf::from(model),
        }
    }

    /// Check whether the compiled executable works.
    pub fn executable_works(&self) -> Result<bool, io::Error> {
        let output = Command::new(&self.model).arg("help").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        Ok(stdout.contains("Bayesian inference with Markov Chain Monte Carlo"))
    }

    /// Call the executable with option "info" and return the result.
    pub fn executable_info(&self) -> Result<process::Output, io::Error> {
        Command::new(&self.model).arg("info").output()
    }

    /// Attempt to compile the Stan model. If successful,
    /// the output (which may be useful for logging) is returned,
    /// otherwise, the error is coarsely categorized and returned.
    pub fn compile(&self) -> Result<process::Output, CompilationError> {
        self.compile_with_args::<[_; 0], &str>([])
    }

    /// Attempt to compile the Stan model, passing the given `args` on to
    /// `make`. If successful, the output (which may be useful for logging) is returned,
    /// otherwise, the error is coarsely categorized and returned.
    pub fn compile_with_args<I, S>(&self, args: I) -> Result<process::Output, CompilationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<ffi::OsStr>,
    {
        if self.is_workspace_dirty() {
            self.try_remove_executable()?;
        }
        let _ = self.validate_cmdstan()?;
        self.make(args)
    }

    /// Is the workspace dirty? (i.e. is there a pre-existing executable?)
    fn is_workspace_dirty(&self) -> bool {
        self.model.exists()
    }
    /// Try to remove the executable file.
    fn try_remove_executable(&self) -> Result<(), CompilationError> {
        fs::remove_file(&self.model).map_err(|e| DirtyWorkspaceError(e))
    }

    /// Assuming that `self.cmdstan` is a working `CmdStan` installation,
    /// call `make` with the supplied arguments.  Not intended for public API.
    fn make<I, S>(&self, args: I) -> Result<process::Output, CompilationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<ffi::OsStr>,
    {
        match Command::new(MAKE)
            .current_dir(&self.cmdstan)
            .args(args)
            .arg(&self.model)
            .output()
        {
            Ok(output) => {
                if self.executable_works().unwrap_or(false) {
                    Ok(output)
                } else {
                    Err(StanCompilerError(output))
                }
            }
            Err(e) => Err(ProcessError(e)),
        }
    }

    /// Is `self.cmdstan` a working `CmdStan` installation?
    fn validate_cmdstan(&self) -> Result<(), CompilationError> {
        match Command::new(MAKE).current_dir(&self.cmdstan).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout[..]);
                if !stdout.contains("Build a Stan program") {
                    Err(MakeError(format!(
                        "Unexpected behavior of `{}` in {:?}",
                        MAKE, &self.cmdstan
                    )))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(ProcessError(e)),
        }
    }

    /// Call the executable with the arguments given by `arg_tree`.
    pub fn call_executable(&self, arg_tree: &ArgumentTree) -> Result<process::Output, io::Error> {
        Command::new(&self.model)
            .args(arg_tree.command_string().split_whitespace())
            .output()
    }

    /// Read in and analyze the output of one or more Markov chains to
    /// check for potential problems.  See
    /// <https://mc-stan.org/docs/cmdstan-guide/diagnose.html> for
    /// more information.
    pub fn diagnose(&self, arg_tree: &ArgumentTree) -> Result<process::Output, io::Error> {
        let files = arg_tree.output_files();
        let mut path = PathBuf::from(&self.cmdstan);
        path.push("bin");
        path.push("diagnose");
        Command::new(path).args(files.into_iter()).output()
    }

    /// Report statistics for one or more Stan csv files from a HMC
    /// sampler run.  See
    /// <https://mc-stan.org/docs/cmdstan-guide/stansummary.html> for
    /// more information.
    pub fn stansummary(
        &self,
        arg_tree: &ArgumentTree,
        opts: Option<StanSummaryOptions>,
    ) -> Result<process::Output, io::Error> {
        let files = arg_tree.output_files();
        let mut path = PathBuf::from(&self.cmdstan);
        path.push("bin");
        path.push("stansummary");
        match opts {
            Some(opts) => Command::new(path)
                .args(files.into_iter())
                .args(opts.command_fragment().split_whitespace())
                .output(),
            None => Command::new(path).args(files.into_iter()).output(),
        }
    }

    pub fn cmdstan<'a>(&'a self) -> &'a Path {
        &self.cmdstan
    }
}

/// Options for the `stansummary` tool. See
/// <https://mc-stan.org/docs/cmdstan-guide/stansummary.html> for more
/// information.
#[derive(Debug, PartialEq, Clone)]
pub struct StanSummaryOptions {
    /// Display the chain autocorrelation for the n-th input file, in
    /// addition to statistics.
    pub autocorr: Option<i32>,
    /// Write statistics to a csv file.
    pub csv_filename: Option<String>,
    /// Percentiles to report as ordered set of comma-separated
    /// integers from (1,99), inclusive. Default is 5,50,95.
    pub percentiles: Option<Vec<u8>>,
    /// Significant figures reported. Default is 2. Must be an integer
    /// from (1, 18), inclusive.
    pub sig_figs: Option<u8>,
}
impl StanSummaryOptions {
    pub fn new() -> Self {
        Self {
            autocorr: None,
            csv_filename: None,
            percentiles: None,
            sig_figs: None,
        }
    }
    pub fn command_fragment(&self) -> String {
        let mut s = String::new();
        let mut state = false;
        match &self.autocorr {
            Some(n) => {
                state = true;
                write!(&mut s, "--autocorr {}", n).unwrap();
            }
            None => (),
        }
        match &self.csv_filename {
            Some(file) => {
                if state {
                    write!(&mut s, " --csv_filename {}", file).unwrap();
                } else {
                    state = true;
                    write!(&mut s, "--csv_filename {}", file).unwrap();
                }
            }
            None => (),
        }
        match &self.percentiles {
            Some(values) => {
                if state {
                    write!(&mut s, " --percentiles ").unwrap();
                } else {
                    state = true;
                    write!(&mut s, "--percentiles ").unwrap();
                }
                let mut values = values.iter();
                if let Some(val) = values.next() {
                    write!(&mut s, "{}", val).unwrap();
                }
                while let Some(val) = values.next() {
                    write!(&mut s, ",{}", val).unwrap();
                }
            }
            None => (),
        }
        match &self.sig_figs {
            Some(n) => {
                if state {
                    write!(&mut s, " --sig_figs {}", n).unwrap();
                } else {
                    write!(&mut s, "--sig_figs {}", n).unwrap();
                }
            }
            None => (),
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod stansummary_options {
        use super::*;

        #[test]
        fn command_fragment() {
            let x = StanSummaryOptions {
                autocorr: None,
                csv_filename: Some("stansummary.csv".to_string()),
                percentiles: Some(vec![5, 25, 50, 75, 95]),
                sig_figs: Some(6),
            };
            assert_eq!(
                x.command_fragment(),
                "--csv_filename stansummary.csv --percentiles 5,25,50,75,95 --sig_figs 6"
            );

            let x = StanSummaryOptions {
                autocorr: Some(1),
                csv_filename: None,
                percentiles: Some(vec![50, 75]),
                sig_figs: None,
            };
            assert_eq!(x.command_fragment(), "--autocorr 1 --percentiles 50,75");

            let x = StanSummaryOptions {
                autocorr: Some(1),
                csv_filename: Some("hello.csv".to_string()),
                percentiles: Some(vec![50]),
                sig_figs: None,
            };
            assert_eq!(
                x.command_fragment(),
                "--autocorr 1 --csv_filename hello.csv --percentiles 50"
            );

            let x = StanSummaryOptions {
                autocorr: None,
                csv_filename: Some("hello.csv".to_string()),
                percentiles: Some(vec![50]),
                sig_figs: Some(3),
            };
            assert_eq!(
                x.command_fragment(),
                "--csv_filename hello.csv --percentiles 50 --sig_figs 3"
            );
        }
    }
}
