use crate::constants::*;
use std::{
    error, fmt,
    hash::Hash,
    io,
    process::{self},
};

#[derive(Debug)]
pub struct Error {
    pub(crate) kind: ErrorKind,
    pub(crate) repr: Repr,
}
impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    pub(crate) fn new(kind: ErrorKind, repr: Repr) -> Self {
        Self { kind, repr }
    }

    pub(crate) fn appears_ok(
        kind: ErrorKind,
        output: process::Output,
        // needle: &'static str,
    ) -> Result<(), Self> {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            if !stdout.contains(kind.needle()) {
                Err(Self::new(kind, output.into()))
            } else {
                Ok(())
            }
        } else {
            Err(Self::new(kind, output.into()))
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.as_str(), &self.repr)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.repr {
            Repr::Io(e) => Some(e),
            Repr::UnsuccessfulExit(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    Bernoulli,
    Compilation,
    Diagnose,
    Executable,
    Install,
    Make,
    ModelFile,
    StanC,
    StanSummary,
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        use ErrorKind::*;
        match self {
            Bernoulli => MAKE_BERNOULLI,
            Compilation => MAKE,
            Diagnose => MAKE_DIAGNOSE,
            Executable => "model executable",
            Install => "cmdstan install",
            Make => MAKE,
            ModelFile => "model file",
            StanC => MAKE_STANC,
            StanSummary => MAKE_STANSUMMARY,
        }
    }
    /// Not every kind has a meaningful needle with which to probe
    /// stdout or stderr. These only make sense to use in the context
    /// verifying invariants.
    fn needle(&self) -> &'static str {
        use ErrorKind::*;
        match self {
            // While the rest contain text which should appear
            // when called with no arguments, the following
            // appears when the executable is called with valid arguments
            // -- part of the initialization test.
            Bernoulli => "Adjust your expectations accordingly!",
            Compilation => "",
            Diagnose => "diagnose <filename 1>",
            Executable => "Bayesian inference with Markov Chain Monte Carlo",
            Install => "",
            Make => "Build CmdStan utilities",
            ModelFile => "",
            StanC => "stanc [option]",
            StanSummary => "stansummary [OPTIONS]",
        }
    }
}
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub(crate) enum Repr {
    Io(io::Error),
    UnsuccessfulExit(process::Output),
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => fmt::Display::fmt(e, f),
            Self::UnsuccessfulExit(_) => f.write_str("process exit status not zero"),
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => fmt::Debug::fmt(e, f),
            Self::UnsuccessfulExit(output) => f
                .debug_struct("UnsuccessfulExit")
                .field("stdout", &String::from_utf8_lossy(&output.stdout[..]))
                .field("stderr", &String::from_utf8_lossy(&output.stderr[..]))
                .finish(),
        }
    }
}

impl From<io::Error> for Repr {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<process::Output> for Repr {
    fn from(output: process::Output) -> Self {
        Self::UnsuccessfulExit(output)
    }
}
