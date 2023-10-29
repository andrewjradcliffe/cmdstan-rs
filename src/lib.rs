use argument_tree::ArgumentTree;
use std::fmt::Write;
use std::process::{self, Command};
use std::{env, ffi, fs, io, path::Path, path::PathBuf, str};
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

/// Stan programs may be provided using either a string containing the
/// Stan code, or a path to a file. Rather than pun on a string to
/// handle both cases, this enum serves as a gatekeeper so that inputs
/// are well-formed and unambiguous.  `From` implementations provide
/// nearly the same ergonomics as would be achieved by punning on a
/// string, but without sacrificing clarity of intent.
#[derive(Debug, PartialEq, Clone)]
pub enum StanProgram {
    Code(String),
    File(PathBuf),
}
impl From<&str> for StanProgram {
    fn from(code: &str) -> Self {
        StanProgram::Code(code.to_string())
    }
}
impl From<&Path> for StanProgram {
    fn from(path: &Path) -> Self {
        StanProgram::File(path.to_path_buf())
    }
}
impl From<String> for StanProgram {
    fn from(code: String) -> Self {
        StanProgram::Code(code)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Model {
    pub model_name: String,
    pub workspace: String,
    pub stan_program: StanProgram,
}
impl Model {
    pub fn setup(&self) -> io::Result<()> {
        fs::create_dir_all(&self.workspace)?;
        let mut path = PathBuf::from(&self.workspace);
        path.push(&self.model_name);
        path.set_extension("stan");
        match &self.stan_program {
            StanProgram::Code(code) => {
                fs::write(path, code)?;
            }
            StanProgram::File(file) => {
                let code = fs::read_to_string(file)?;
                fs::write(path, code)?;
            }
        }
        Ok(())
    }
    pub fn model(&self) -> String {
        let mut path = PathBuf::from(&self.workspace);
        path.push(&self.model_name);
        path.to_string_lossy().to_string()
    }
}

// Relying on environment variable is not desirable.
// impl TryFrom<Model> for Control {
//     type Error = env::VarError;
//     fn try_from(model: Model) -> Result<Self, Self::Error> {
//         let cmdstan_home = env::var("CMDSTAN_HOME")?;
//         let model = model.model();
//         Ok(Self {
//             cmdstan_home,
//             model,
//         })
//     }
// }

// #[derive(Debug, PartialEq, Clone)]
// pub struct ModelBuilder {
//     model_name: Option<String>,
//     workspace: Option<String>,
//     stan_program: Option<String>,
// }
// impl ModelBuilder {
//     pub fn new() -> Self {
//         Self {
//             model_name: None,
//             workspace: None,
//             stan_program: None,
//         }
//     }
//     insert_field!(model_name, String);
//     insert_field!(workspace, String);
//     insert_field!(stan_program, String);
//     pub fn build(self) -> Model {
//         let model_name = self.model_name.unwrap_or_else(|| "model".to_string());
//         let workspace = self
//             .workspace
//             .unwrap_or_else(|| env::current_dir().unwrap().to_str().unwrap().to_string());
//         let stan_program = self.stan_program.unwrap_or_else(|| "".to_string());
//         Model {
//             model_name,
//             workspace,
//             stan_program,
//         }
//     }
// }

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

// #[derive(Debug, PartialEq)]
// pub struct ToolControl {
//     cmdstan_home: String,
//     workspace: String,
// }
// impl ToolControl {
//     pub fn new(cmdstan_home: &str, workspace: &str) -> Self {
//         Self {
//             cmdstan_home: cmdstan_home.to_string(),
//             workspace: workspace.to_string(),
//         }
//     }
//     // pub fn diagnose(&self, arg_tree: &ArgumentTree) -> Result<process::Output, io::Error> {
//     //     let files: Vec<PathBuf> = arg_tree
//     //         .output_files()
//     //         .into_iter()
//     //         .map(|file_name| {
//     //             let mut path = PathBuf::from(&self.workspace);
//     //             path.push(file_name);
//     //             path
//     //         })
//     //         .collect();
//     //     let mut path = PathBuf::from(&self.cmdstan_home);
//     //     path.push("bin");
//     //     path.push("diagnose");
//     //     Command::new(path).args(files.into_iter()).output()
//     // }

//     // Alternate option focused on workspace
// }

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
    // insert_field!(autocorr, i32);
    // insert_field!(csv_filename, String);
    // insert_field!(percentiles, Vec<u8>);
    // insert_field!(sig_figs, u8);
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
