use std::{fs, io, path::Path, path::PathBuf};

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
impl From<PathBuf> for StanProgram {
    fn from(pathbuf: PathBuf) -> Self {
        StanProgram::File(pathbuf)
    }
}

/// Information to build a workspace for use with `CmdStan`.
#[derive(Debug, PartialEq, Clone)]
pub struct Workspace {
    /// Name of the model.
    pub model_name: String,
    /// Directory in which executable may be built.
    pub directory: String,
    /// The Stan program which will be compiled to C++, then to an
    /// executable; will be written to a file in `directory` for
    /// posterity.
    pub stan_program: StanProgram,
}
impl Workspace {
    /// Set up the workspace.
    pub fn setup(&self) -> io::Result<()> {
        fs::create_dir_all(&self.directory)?;
        let mut path = PathBuf::from(&self.directory);
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
    /// Return the model target; this may serve as the input to
    /// `Control::new`.
    pub fn model(&self) -> String {
        let mut path = PathBuf::from(&self.directory);
        path.push(&self.model_name);
        path.to_string_lossy().to_string()
    }
}
