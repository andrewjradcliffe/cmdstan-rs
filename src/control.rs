use crate::argument_tree::ArgumentTree;
use std::fmt::Write;
use std::process::{self, Command};
use std::{env, ffi, fs, io, path::Path, path::PathBuf, str};
use thiserror::Error;

/// Structure to direct compilation and execution of a Stan model.
/// Computation of diagnostics and summaries for said model are
/// facilitated through the same interface.
#[derive(Debug, PartialEq, Clone)]
pub struct Control {
    cmdstan_home: String,
    model: String,
}

#[derive(Error, Debug)]
pub enum CompilationError {
    #[error("Change directory failed: {0:?}")]
    ChangeDirectoryError(io::Error),
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

impl Control {
    /// Construct a new instance from a path (`cmdstan_home`) to a `CmdStan`
    /// installation and a path (`model`) to a Stan program.
    pub fn new(cmdstan_home: &str, model: &str) -> Self {
        Self {
            cmdstan_home: cmdstan_home.to_string(),
            model: model.to_string(),
        }
    }

    /// Check whether the compiled executable works.
    pub fn executable_works(&self) -> Result<bool, io::Error> {
        let output = Command::new(&self.model).arg("help").output()?;
        let stdout = str::from_utf8(&output.stdout[..]).unwrap();
        Ok(stdout.contains("Bayesian inference with Markov Chain Monte Carlo"))
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
        match env::set_current_dir(&self.cmdstan_home) {
            Ok(()) => (),
            Err(e) => return Err(ChangeDirectoryError(e)),
        }

        self.check_cmdstan_dir()?;
        self.make(args)
    }

    fn is_workspace_dirty(&self) -> bool {
        let path: &Path = self.model.as_ref();
        path.exists()
    }
    fn try_remove_executable(&self) -> Result<(), CompilationError> {
        match fs::remove_file(&self.model) {
            Ok(()) => Ok(()),
            Err(e) => Err(DirtyWorkspaceError(e)),
        }
    }

    fn make<I, S>(&self, args: I) -> Result<process::Output, CompilationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<ffi::OsStr>,
    {
        match Command::new("make").args(args).arg(&self.model).output() {
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

    fn check_cmdstan_dir(&self) -> Result<(), CompilationError> {
        match Command::new("make").output() {
            Ok(output) => {
                let stdout = str::from_utf8(&output.stdout[..]).unwrap();
                if !stdout.contains("Build a Stan program") {
                    Err(MakeError(format!(
                        "Unexpected behavior of `make` in {}",
                        &self.cmdstan_home
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
        let path: &Path = self.model.as_ref();
        env::set_current_dir(path.parent().unwrap())?;
        Command::new(&self.model)
            .args(arg_tree.command_string().split_whitespace())
            .output()
    }

    /// Read in and analyze the output of one or more Markov chains to
    /// check for potential problems.  See
    /// https://mc-stan.org/docs/cmdstan-guide/diagnose.html for more
    /// information.
    pub fn diagnose(&self, arg_tree: &ArgumentTree) -> Result<process::Output, io::Error> {
        let path: &Path = self.model.as_ref();
        env::set_current_dir(path.parent().unwrap())?;
        let files = arg_tree.output_files();
        let mut path = PathBuf::from(&self.cmdstan_home);
        path.push("bin");
        path.push("diagnose");
        Command::new(path).args(files.into_iter()).output()
    }

    /// Report statistics for one or more Stan csv files from a HMC
    /// sampler run.  See
    /// https://mc-stan.org/docs/cmdstan-guide/stansummary.html for
    /// more information.
    pub fn stansummary(
        &self,
        arg_tree: &ArgumentTree,
        opts: Option<StanSummaryOptions>,
    ) -> Result<process::Output, io::Error> {
        let path: &Path = self.model.as_ref();
        env::set_current_dir(path.parent().unwrap())?;
        let files = arg_tree.output_files();
        let mut path = PathBuf::from(&self.cmdstan_home);
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
}

/// Options for the `stansummary` tool. See
/// https://mc-stan.org/docs/cmdstan-guide/stansummary.html for more
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
    fn command_fragment(&self) -> String {
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
