use std::io::Write;
use std::process::{self, Command};
use std::{env, ffi, fs::File, io, path::Path, path::PathBuf};

#[macro_use]
mod internal_macros;

pub mod argument_tree;
pub mod control;
pub mod diagnose;
pub mod generate_quantities;
pub mod laplace;
pub mod logprob;
pub mod method;
pub mod optimize;
pub mod pathfinder;
pub mod sample;
pub mod variational;

pub use crate::method::*;

use crate::argument_tree::ArgumentTree;
use crate::control::Control;
pub use crate::control::{CompilationError, StanSummaryOptions};

/// A high level interface to construct a model
pub struct CmdStanModel {
    control: Control,
}
impl CmdStanModel {
    /// Construct a new instance from a path (`cmdstan`) to a
    /// `CmdStan` installation and a path to a Stan program.
    pub fn new<P1, P2>(cmdstan: P1, stan_file: P2) -> Self
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let cmdstan: &Path = cmdstan.as_ref();
        let stan_file: &Path = stan_file.as_ref();
        if stan_file.extension().is_some_and(|ext| ext == "stan") {
            Self {
                control: Control::new(cmdstan, stan_file.with_extension("").as_ref()),
            }
        } else {
            Self {
                control: Control::new(cmdstan, stan_file),
            }
        }
    }

    /// Call the executable with the arguments given by `arg_tree`.
    /// On success, returns a snapshot which contains a full record of
    /// the `CmdStan` call.
    pub fn call_executable(&self, arg_tree: &ArgumentTree) -> io::Result<CmdStanOutput> {
        let cwd_at_call = env::current_dir()?;
        let output = self.control.call_executable(arg_tree)?;
        Ok(CmdStanOutput {
            cwd_at_call,
            output,
            argument_tree: arg_tree.clone(),
            cmdstan: self.control.cmdstan().to_path_buf(),
        })
    }

    /// Attempt to compile the Stan model. If successful, the output
    /// (which may be useful for logging) is returned, otherwise, the
    /// error is coarsely categorized and returned.
    pub fn compile(&self) -> Result<process::Output, CompilationError> {
        self.control.compile()
    }

    /// Attempt to compile the Stan model, passing the given `args` on
    /// to `make`. If successful, the output (which may be useful for
    /// logging) is returned, otherwise, the error is coarsely
    /// categorized and returned.
    pub fn compile_with_args<I, S>(&self, args: I) -> Result<process::Output, CompilationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<ffi::OsStr>,
    {
        self.control.compile_with_args(args)
    }

    /// Check whether the compiled executable works.
    pub fn executable_works(&self) -> io::Result<bool> {
        self.control.executable_works()
    }

    /// Call the executable with option "info" and return the result.
    pub fn executable_info(&self) -> io::Result<process::Output> {
        self.control.executable_info()
    }
}

/// A snapshot produced by performing `call_executable` on
/// `CmdStanModel`.  This is a self-contained record, the contents of
/// which include:
///
/// - the console output (exit status, stdout and stderr),
/// - the argument tree with the call was made
/// - the current working directory of the process at the time the call was made
/// - the `CmdStan` installation from the parent `CmdStanModel`
pub struct CmdStanOutput {
    /// Enables methods such as `output_files`, `diagnostic_files`,
    /// etc. to return absolute paths by introspection of the
    /// `ArgumentTree` and `cwd_at_call`.  In essence, if the
    /// output/diagnostic/profile file is relative, then it should be
    /// pushed onto `cwd_at_call`.
    cwd_at_call: PathBuf,
    output: process::Output,
    argument_tree: ArgumentTree,
    cmdstan: PathBuf,
}
impl CmdStanOutput {
    /// Convert files to absolute paths. If the file is already
    /// absolute, this is a no-op (simple move into `PathBuf`);
    /// otherwise the current working directory at the time this
    /// `CmdStanOutput` instance was created serves as the prefix onto
    /// which the relative path will be joined.
    fn files<F>(&self, f: F) -> Vec<PathBuf>
    where
        F: Fn(&ArgumentTree) -> Vec<String>,
    {
        f(&self.argument_tree)
            .into_iter()
            .map(|s| {
                let file: &Path = s.as_ref();
                if file.is_relative() {
                    self.cwd_at_call.join(file)
                } else {
                    PathBuf::from(s)
                }
                // Equivalent, but wasteful if path is already absolute
                // as a copy of `s` would occur.
                // self.cwd_at_call.join(s)
            })
            .collect()
    }
    /// Return the output files associated with the `CmdStan` call.
    pub fn output_files(&self) -> Vec<PathBuf> {
        self.files(|tree| tree.output_files())
    }
    /// Return the diagnostic files associated with the `CmdStan` call.
    pub fn diagnostic_files(&self) -> Vec<PathBuf> {
        self.files(|tree| tree.diagnostic_files())
    }

    /// Return an immutable reference to console output associated
    /// with the `CmdStan` call.
    pub fn output<'a>(&'a self) -> &'a process::Output {
        &self.output
    }

    /// Write the console output to a file.  The path at which the
    /// file will be created is determined in the following manner: If
    /// `file` is `None`, "log.txt" is adjoined on to `cwd_at_call`.
    /// If `file` is `Some(path)` and `path` is a relative path, then
    /// `path` is adjoined on to `cwd_at_call`; otherwise, `path` is
    /// assumed to be an absolute path and is used without
    /// modification. Upon successful write, the path at which the
    /// file was created is returned.
    pub fn write_output<P: AsRef<Path>>(&self, file: Option<P>) -> io::Result<PathBuf> {
        let path = match file {
            Some(file) => {
                let path: &Path = file.as_ref();
                if path.is_relative() {
                    self.cwd_at_call.join(path)
                } else {
                    path.to_path_buf()
                }
            }
            None => self.cwd_at_call.join("log.txt"),
        };
        let mut file = File::create(&path)?;
        file.write_all(&self.output.stdout)?;
        file.write_all(&self.output.stderr)?;
        Ok(path)
    }

    /// Return a reference to the `CmdStan` installation associated
    /// the call.
    pub fn cmdstan<'a>(&'a self) -> &'a Path {
        &self.cmdstan
    }
    /// Return a reference to the current working directory at the
    /// time of the call.
    pub fn cwd_at_call<'a>(&'a self) -> &'a Path {
        &self.cwd_at_call
    }

    /// Read in and analyze the output of one or more Markov chains to
    /// check for potential problems.  See
    /// <https://mc-stan.org/docs/cmdstan-guide/diagnose.html> for
    /// more information.
    pub fn diagnose(&self) -> io::Result<process::Output> {
        let mut path = PathBuf::from(&self.cmdstan);
        path.push("bin");
        path.push("diagnose");
        Command::new(path).args(self.output_files()).output()
    }
    /// Report statistics for one or more Stan csv files from a HMC
    /// sampler run.  See
    /// <https://mc-stan.org/docs/cmdstan-guide/stansummary.html> for
    /// more information.
    pub fn stansummary(&self, opts: Option<StanSummaryOptions>) -> io::Result<process::Output> {
        let mut path = PathBuf::from(&self.cmdstan);
        path.push("bin");
        path.push("stansummary");
        let mut cmd = Command::new(path);
        cmd.args(self.output_files());
        match opts {
            Some(opts) => cmd
                .args(opts.command_fragment().split_whitespace())
                .output(),
            None => cmd.output(),
        }
    }
}
