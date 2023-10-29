use argument_tree::ArgumentTree;
use std::process::{self, Command};
use std::{env, ffi, fs, io, path::Path, str};
use thiserror::Error;

#[macro_use]
mod internal_macros;

pub mod argument_tree;
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

#[derive(Debug, PartialEq)]
pub struct Control {
    cmdstan_home: String,
    // workspace: &str,
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
    pub fn new(cmdstan_home: &str, model: &str) -> Self {
        Self {
            cmdstan_home: cmdstan_home.to_string(),
            model: model.to_string(),
        }
    }

    pub fn executable_works(&self) -> Result<bool, io::Error> {
        let output = Command::new(&self.model).arg("help").output()?;
        let stdout = str::from_utf8(&output.stdout[..]).unwrap();
        Ok(stdout.contains("Bayesian inference with Markov Chain Monte Carlo"))
    }

    /// Attempt to compile the Stan model. If successful,
    /// the output is returned (it may be useful for logging),
    /// otherwise, the error is coarsely categorized and returned.
    pub fn compile(&self) -> Result<process::Output, CompilationError> {
        self.compile_with_args::<[_; 0], &str>([])
    }

    /// Attempt to compile the Stan model, passing the given `args` on to
    /// `make`. If successful, the output is returned (it may be useful for logging),
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
    // fn try_clean_workspace(&self) -> Result<(), CompilationError> {
    //     if self.is_workspace_dirty() {
    //         self.try_remove_executable()
    //     } else {
    //         Ok(())
    //     }
    // }

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

    pub fn call_executable(&self, arg_tree: &ArgumentTree) -> Result<process::Output, io::Error> {
        env::set_current_dir(&self.model.rsplit_once('/').unwrap().0)?;
        Command::new(&self.model)
            .args(arg_tree.command_string().split_whitespace())
            .output()
    }
}
