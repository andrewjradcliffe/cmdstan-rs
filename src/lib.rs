use std::env;
use std::ffi;
use std::io;
use std::process::Command;
use std::str;
// use std::process::ExitStatus;
use std::process;
use std::str::FromStr;
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

// pub struct Model {
//     stan_file: String,
//     outdir: String,
// }

#[derive(Debug, PartialEq)]
pub struct Control {
    cmdstan_home: String,
    // executable: String,
    // workspace: &str,
    model: String,
}
// This shouldn't be implemented
impl FromStr for Control {
    type Err = env::VarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let model = s.to_string();
        // let workspace = model.as_str().rsplit_once('/').unwrap().0;
        let cmdstan_home = env::var("CMDSTAN_HOME")?;
        Ok(Self {
            cmdstan_home,
            model,
        })
    }
}

impl From<(&str, &str)> for Control {
    fn from((model, cmdstan_home): (&str, &str)) -> Self {
        Self {
            model: model.to_string(),
            cmdstan_home: cmdstan_home.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum CompilationError {
    #[error("Change directory failed")]
    ChangeDirectoryError(#[from] io::Error),
    #[error("Something happened to the process")]
    ProcessError(io::Error),
    #[error("Make problem")]
    MakeError(String),
    #[error("Compilation problem: {0:?}")]
    StanCompilerError(process::Output),
}
use CompilationError::*;

impl Control {
    /// Attempt to compile the Stan model. If successful,
    /// the output is returned (it may be useful for logging),
    /// otherwise, the error is coarsely categorized and returned.
    pub fn compile(&self) -> Result<process::Output, CompilationError> {
        match env::set_current_dir(&self.cmdstan_home) {
            Ok(()) => (),
            Err(e) => return Err(ChangeDirectoryError(e)),
        }

        self.check_cmdstan_dir()?;
        let attempt = self.make::<[_; 0], &str>([]);
        match attempt {
            Ok(val) => Ok(val),
            Err(StanCompilerError(e)) => {
                let attempt2 = self.make::<[_; 0], &str>([]);
                match attempt2 {
                    Ok(_) => Ok(e),
                    Err(_) => Err(StanCompilerError(e)),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn executable_works(&self) -> Result<bool, io::Error> {
        let output = Command::new(&self.model).arg("help").output()?;
        let stdout = str::from_utf8(&output.stdout[..]).unwrap();
        Ok(stdout.contains("Bayesian inference with Markov Chain Monte Carlo"))
    }

    pub fn compile2(&self) -> Result<process::Output, CompilationError> {
        match env::set_current_dir(&self.cmdstan_home) {
            Ok(()) => (),
            Err(e) => return Err(ChangeDirectoryError(e)),
        }

        self.check_cmdstan_dir()?;
        self.make2::<[_; 0], &str>([])
    }

    /// Attempt to compile the Stan model, passing the given `args` on to
    /// `make`. If successful, the output is returned (it may be useful for logging),
    /// otherwise, the error is coarsely categorized and returned.
    pub fn compile_with_args<I, S>(&self, args: I) -> Result<process::Output, CompilationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<ffi::OsStr>,
    {
        match env::set_current_dir(&self.cmdstan_home) {
            Ok(()) => (),
            Err(e) => return Err(ChangeDirectoryError(e)),
        }

        self.check_cmdstan_dir()?;
        self.make2(args)
    }

    fn make<I, S>(&self, args: I) -> Result<process::Output, CompilationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<ffi::OsStr>,
    {
        match Command::new("make").args(args).arg(&self.model).output() {
            Ok(output) => {
                let stdout = str::from_utf8(&output.stdout[..]).unwrap();
                if stdout.contains("is up to date.\n") {
                    Ok(output)
                } else {
                    Err(StanCompilerError(output))
                }
            }
            Err(e) => Err(ProcessError(e)),
        }
    }

    fn make2<I, S>(&self, args: I) -> Result<process::Output, CompilationError>
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
}
