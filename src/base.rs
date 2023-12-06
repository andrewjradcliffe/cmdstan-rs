use std::{
    convert::TryFrom,
    env,
    ffi::OsStr,
    fmt,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self, Command},
    sync::{Arc, RwLock},
};
use thiserror::Error;

/// Try to determine if the file exists by attempting to open it in read-only mode.
fn try_open<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    File::open(path).map(|_| ())
}

fn try_exec<S: AsRef<OsStr>>(path: S) -> Result<process::Output, io::Error> {
    Command::new(path).output()
}

fn success_and_contains(
    output: process::Output,
    needle: &'static str,
    kind: ErrorKind,
) -> Result<(), Error> {
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if !stdout.contains(needle) {
            return Err(Error::new_empty(kind));
        };
        Ok(())
    } else {
        return Err(Error::new_empty(kind));
    }
}

#[derive(Debug)]
pub enum StanProgramError {
    Io(io::Error),
    Ext,
}
impl From<io::Error> for StanProgramError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl fmt::Display for StanProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ext => write!(f, "path extension must be stan"),
            Self::Io(e) => write!(f, "{}", e),
        }
    }
}

pub struct StanProgram {
    path: PathBuf,
}

impl TryFrom<&Path> for StanProgram {
    type Error = StanProgramError;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        match path.extension() {
            Some(ext) if ext == "stan" => (),
            _ => return Err(Self::Error::Ext),
        }
        // Open is the most reliable way to determine if the file exists
        // and can be read.
        File::open(path)?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

static BERNOULLI: &str = "bernoulli";
static STANC: &str = "stanc";
static STANSUMMARY: &str = "stansummary";
static DIAGNOSE: &str = "diagnose";

#[cfg(unix)]
static MAKE: &str = "make";
#[cfg(windows)]
static MAKE: &str = "mingw32-make";

#[cfg(unix)]
static OS_EXE_EXT: &str = "";
#[cfg(windows)]
static OS_EXE_EXT: &str = "exe";

#[cfg(unix)]
static MAKE_BERNOULLI: &str = "examples/bernoulli/bernoulli";
#[cfg(windows)]
static MAKE_BERNOULLI: &str = "examples/bernoulli/bernoulli.exe";

#[cfg(unix)]
static MAKE_STANC: &str = "bin/stanc";
#[cfg(windows)]
static MAKE_STANC: &str = "bin/stanc.exe";

#[cfg(unix)]
static MAKE_STANSUMMARY: &str = "bin/stansummary";
#[cfg(windows)]
static MAKE_STANSUMMARY: &str = "bin/stansummary.exe";

#[cfg(unix)]
static MAKE_DIAGNOSE: &str = "bin/diagnose";
#[cfg(windows)]
static MAKE_DIAGNOSE: &str = "bin/diagnose.exe";

#[derive(Debug)]
pub enum ErrorKind {
    Make,
    StanC,
    StanSummary,
    Diagnose,
    Bernoulli,
    Compilation,
    Install,
    InvalidExecutable,
}

#[derive(Debug, Error)]
#[error("error related to {:?}", .kind)]
pub struct Error {
    kind: ErrorKind,
    #[source]
    source: Option<io::Error>,
}
impl Error {
    fn new(kind: ErrorKind, source: Option<io::Error>) -> Self {
        Self { kind, source }
    }
    /// Convenience constructor; same effect as `From<ErrorKind>`, but
    /// not exposed in public API.
    fn new_empty(kind: ErrorKind) -> Self {
        Self::new(kind, None)
    }
}

/// Path to CmdStan (`root`) directory and paths to binary utilities.
/// This is necessary for locking of the public-facing resources
/// (see `CmdStan` type).
/// It also provides a means for separation of concerns, which helps reduce code
/// repetition. Perhaps perhaps more importantly, it enables the methods and
/// associated functions of the `CmdStan` type to be written with clarity,
/// since any operation must acquire this resource.
#[derive(Debug, Clone, PartialEq)]
struct CmdStanInner {
    root: PathBuf,
    stanc: PathBuf,
    stansummary: PathBuf,
    diagnose: PathBuf,
}

/// Operations to be called only from within a `CmdStan` instance where
/// one has write access to the `inner` field.
/// Alternatively, during `CmdStan::try_from`.
impl CmdStanInner {
    fn try_ensure(&self, bin: &Path, target: &str) -> Result<(), io::Error> {
        match try_open(bin) {
            Ok(_) => Ok(()),
            Err(_) => {
                self.make(target)?;
                try_open(bin)
            }
        }
    }
    fn try_ensure_stanc(&self) -> Result<(), Error> {
        self.try_ensure(&self.stanc, MAKE_STANC)
            .map_err(|e| Error::new(ErrorKind::StanC, Some(e)))
    }
    fn try_ensure_stansummary(&self) -> Result<(), Error> {
        self.try_ensure(&self.stansummary, MAKE_STANSUMMARY)
            .map_err(|e| Error::new(ErrorKind::StanSummary, Some(e)))
    }
    fn try_ensure_diagnose(&self) -> Result<(), Error> {
        self.try_ensure(&self.diagnose, MAKE_DIAGNOSE)
            .map_err(|e| Error::new(ErrorKind::Diagnose, Some(e)))
    }

    fn make<S: AsRef<OsStr>>(&self, arg: S) -> io::Result<process::Output> {
        Command::new(MAKE).current_dir(&self.root).arg(arg).output()
    }
}

impl TryFrom<&Path> for CmdStanInner {
    type Error = Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // A reliable way to determine if the directory exists
        // and is accessible
        fs::read_dir(path).map_err(|e| Self::Error {
            kind: ErrorKind::Install,
            source: Some(e),
        })?;

        // Superficial check for make
        let output = Command::new(MAKE)
            .current_dir(path)
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::Make, Some(e)))?;
        // success_and_contains(output, "Build CmdStan utilities", ErrorKind::Make)?
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            if !stdout.contains("Build CmdStan utilities") {
                return Err(Self::Error::new_empty(ErrorKind::Make));
            };
        } else {
            return Err(Self::Error::new_empty(ErrorKind::Make));
        }

        // Since things appear to work on the surface, initialize
        // and use the stock methods to verify.
        let root = path.to_path_buf();

        let mut stanc = root.clone();
        stanc.push("bin");
        stanc.push(STANC);
        stanc.set_extension(OS_EXE_EXT);

        let mut stansummary = stanc.clone();
        stansummary.pop();
        stansummary.push(STANSUMMARY);
        stansummary.set_extension(OS_EXE_EXT);

        let mut diagnose = stanc.clone();
        diagnose.pop();
        diagnose.push(DIAGNOSE);
        diagnose.set_extension(OS_EXE_EXT);

        let inner = Self {
            root,
            stanc,
            stansummary,
            diagnose,
        };

        inner.try_ensure_stanc()?;
        inner.try_ensure_stansummary()?;
        inner.try_ensure_diagnose()?;

        Ok(inner)
    }
}

#[derive(Debug, Clone)]
pub struct CmdStan {
    inner: Arc<RwLock<CmdStanInner>>,
}

impl TryFrom<&Path> for CmdStan {
    type Error = Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // This includes a few weaker checks
        let mut inner = CmdStanInner::try_from(path)?;

        // Rather than verify individual files, a simple way to
        // verify CmdStan works is to build and run the bernoulli example
        let output = inner
            .make(MAKE_BERNOULLI)
            .map_err(|e| Self::Error::new(ErrorKind::Bernoulli, Some(e)))?;
        if !output.status.success() {
            return Err(Self::Error::new_empty(ErrorKind::Bernoulli));
        }

        // Then, we abuse the root buffer to save an allocation
        let root = &mut inner.root;
        root.push("examples");
        root.push(BERNOULLI);
        root.push(BERNOULLI);
        root.set_extension(OS_EXE_EXT);
        try_open(&root).map_err(|e| Self::Error::new(ErrorKind::Bernoulli, Some(e)))?;

        let output = Command::new(&root)
            .current_dir(path)
            .arg("sample")
            .arg("data")
            .arg("file=examples/bernoulli/bernoulli.data.json")
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::Bernoulli, Some(e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            if !stdout.contains("Adjust your expectations accordingly!") {
                return Err(Self::Error::new_empty(ErrorKind::Bernoulli));
            };
        } else {
            return Err(Self::Error::new_empty(ErrorKind::Bernoulli));
        }

        // Then, we restore the root to its previous state
        root.pop();
        root.pop();
        root.pop();

        let output = Command::new(&inner.stansummary)
            .current_dir(path)
            .arg("output.csv")
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::StanSummary, Some(e)))?;

        if !output.status.success() {
            return Err(Self::Error::new_empty(ErrorKind::StanSummary));
        }

        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if let Some(line) = stdout.lines().find(|l| l.starts_with("theta")) {
            let mut iter = line.split_whitespace();
            let f = |x: &str| x.parse::<f64>().ok();
            let mean = iter.nth(1).and_then(f);
            let stddev = iter.nth(1).and_then(f);
            match (mean, stddev) {
                (Some(mean), Some(stddev)) if mean - stddev < 0.2 && 0.2 < mean + stddev => (),
                _ => return Err(Self::Error::new_empty(ErrorKind::Bernoulli)),
            }
        } else {
            return Err(Self::Error::new_empty(ErrorKind::Bernoulli));
        }

        let output = Command::new(&inner.diagnose)
            .current_dir(path)
            .arg("output.csv")
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::Diagnose, Some(e)))?;
        if !output.status.success() {
            return Err(Self::Error::new_empty(ErrorKind::Diagnose));
        }
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if !stdout.contains("Processing complete, no problems detected") {
            return Err(Self::Error::new_empty(ErrorKind::Bernoulli));
        }

        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
        })
    }
}

impl CmdStan {
    pub fn compile<I, S>(&self, prog: &StanProgram, args: I) -> Result<CmdStanModel, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut exec = prog.path.clone();
        exec.set_extension(OS_EXE_EXT);

        // Compilation has the potential to touch all of the files in
        // the CmdStan directory.
        let guard = self.inner.write().unwrap();

        // We need to detect whether the diagnose and stansummary utilities
        // are deleted. If combined with invalid unicode, it will be difficult
        // to detect whether `clean-all` is actually passed to make --
        // we would hope that make fails.
        let mut state = false;
        let args = args.into_iter().inspect(|os| {
            state |= os
                .as_ref()
                .to_str()
                .is_some_and(|s| s.trim() == "clean-all")
        });

        // This is lazy, but, not unreasonable given the myriad ways in which
        // things can fail.
        let mut cmd = Command::new(MAKE);
        cmd.current_dir(&guard.root).args(args).arg(&exec);

        let output = cmd
            .output()
            .map_err(|e| Error::new(ErrorKind::Compilation, Some(e)))?;

        if !output.status.success() {
            return Err(Error::new_empty(ErrorKind::Compilation));
        }

        // If everything was cleaned, then we need to re-build the utilities
        // in order to maintain the invariants.
        if state {
            guard.try_ensure_stanc()?;
            guard.try_ensure_stansummary()?;
            guard.try_ensure_diagnose()?;
        }

        // Then, we subject the binary to the same tests as are required
        // to construct directly from a path.
        CmdStanModel::try_from(exec.as_ref())
    }
}

pub struct CmdStanModel {
    exec: PathBuf,
}

impl TryFrom<&Path> for CmdStanModel {
    type Error = Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        try_open(path).map_err(|e| Self::Error::new(ErrorKind::InvalidExecutable, Some(e)))?;

        let output = Command::new(path)
            .arg("help")
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::InvalidExecutable, Some(e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            if !stdout.contains("Bayesian inference with Markov Chain Monte Carlo") {
                return Err(Self::Error::new_empty(ErrorKind::InvalidExecutable));
            }
        } else {
            return Err(Self::Error::new_empty(ErrorKind::InvalidExecutable));
        }

        Ok(Self {
            exec: path.to_path_buf(),
        })
    }
}
