use crate::constants::*;
use crate::error::*;
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

/// Try to determine if the file exists by attempting to open it in read-only mode.
fn try_open<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    File::open(path).map(|_| ())
}

/// Try to determine if the binary file exists and is executable by attempting
/// to call it with zero arguments. More expensive than simply opening the file,
/// but we need to verify both properties, thus we would need to do this anyway.
/// In other words, this is a test of the sufficiency condition, whereas
/// `try_open` only tests a necessary condition.
fn try_exec<S: AsRef<OsStr>>(path: S) -> Result<process::Output, io::Error> {
    Command::new(path).output()
}

pub struct StanProgram {
    path: PathBuf,
}

impl TryFrom<&Path> for StanProgram {
    type Error = Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        match path.extension() {
            Some(ext) if ext == "stan" => (),
            _ => {
                return Err(Self::Error::new(
                    ErrorKind::ModelFile,
                    io::Error::other("path extension must be stan").into(),
                ));
            }
        }
        let op = |e: io::Error| Self::Error::new(ErrorKind::ModelFile, e.into());
        // The file must exist at the time of construction, not be
        // a hypothetical path at which a file might later appear.
        // Certainly, to compile the Stan program, the absolute path is necessary.
        // If not canonicalized at the point of creation, then the only other
        // point at which it would be canonicalized is `CmdStan::compile`,
        // but the current directory of the thread when `CmdStan::compile`
        // is called may be different (changed by user, not this library)
        // from the current directory when this instance is created.
        // It is far easier to reason about when the path is frozen (canonicalized)
        // at the point of creation. If a user understand this, then they can plan
        // their relative path shenanigans accordingly.
        let path = fs::canonicalize(path).map_err(op)?;
        try_open(&path).map_err(op)?;
        Ok(Self { path })
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

macro_rules! impl_try_ensure {
    ($F:ident, $target:ident, $kind:expr, $field:ident) => {
        fn $F(&self) -> Result<(), Error> {
            let output = self
                .try_ensure(&self.$field, $target)
                .map_err(|e| Error::new($kind, e.into()))?;
            Error::appears_ok($kind, output)
        }
    };
}

/// Operations to be called only from within a `CmdStan` instance where
/// one has write access to the `inner` field.
/// Alternatively, during `CmdStanInner::try_from` or `CmdStan::try_from`
impl CmdStanInner {
    fn try_ensure(&self, bin: &Path, target: &'static str) -> Result<process::Output, io::Error> {
        match try_exec(bin) {
            Ok(x) => Ok(x),
            Err(_) => {
                self.make(target)?;
                try_exec(bin)
            }
        }
    }
    impl_try_ensure!(try_ensure_stanc, MAKE_STANC, ErrorKind::StanC, stanc);
    impl_try_ensure!(
        try_ensure_stansummary,
        MAKE_STANSUMMARY,
        ErrorKind::StanSummary,
        stansummary
    );
    impl_try_ensure!(
        try_ensure_diagnose,
        MAKE_DIAGNOSE,
        ErrorKind::Diagnose,
        diagnose
    );

    fn make<S: AsRef<OsStr>>(&self, arg: S) -> io::Result<process::Output> {
        Command::new(MAKE).current_dir(&self.root).arg(arg).output()
    }
}

impl TryFrom<&Path> for CmdStanInner {
    type Error = Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let install_err = |e: io::Error| Self::Error::new(ErrorKind::Install, e.into());
        // A key invariant is that `CmdStan` can work from anywhere,
        // thus, we need an absolute path for the proposed root.
        // All subsequent invariants will be established on the basis
        // of the root, thus, we must first obtain a canonicalized absolute pathname.
        let root = fs::canonicalize(path).map_err(install_err)?;

        // A reliable way to determine if the directory exists
        // and is accessible is to attempt to read it.
        fs::read_dir(&root).map_err(install_err)?;

        // Superficial check for make
        let output = Command::new(MAKE)
            .current_dir(&root)
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::Make, e.into()))?;
        Self::Error::appears_ok(ErrorKind::Make, output)?;

        // Since things appear to work on the surface, initialize
        // and use the stock methods to verify.
        let mut stanc = root.clone();
        stanc.push("bin");
        stanc.push(STANC);

        let mut stansummary = stanc.clone();
        stansummary.pop();
        stansummary.push(STANSUMMARY);

        let mut diagnose = stanc.clone();
        diagnose.pop();
        diagnose.push(DIAGNOSE);

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
        let inner = CmdStanInner::try_from(path)?;

        // Rather than verify individual files, a simple way to
        // verify CmdStan works is to build and run the bernoulli example
        let output = inner
            .make(MAKE_BERNOULLI)
            .map_err(|e| Self::Error::new(ErrorKind::Bernoulli, e.into()))?;
        if !output.status.success() {
            return Err(Self::Error::new(ErrorKind::Bernoulli, output.into()));
        }

        let mut exec = inner.root.clone();
        exec.push("examples");
        exec.push("bernoulli");
        exec.push("bernoulli");
        exec.set_extension(OS_EXE_EXT);
        try_open(&exec).map_err(|e| Self::Error::new(ErrorKind::Bernoulli, e.into()))?;

        let output = Command::new(&exec)
            .current_dir(&inner.root)
            .arg("sample")
            .arg("data")
            .arg("file=examples/bernoulli/bernoulli.data.json")
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::Bernoulli, e.into()))?;
        Error::appears_ok(ErrorKind::Bernoulli, output)?;

        let output = Command::new(&inner.stansummary)
            .current_dir(&inner.root)
            .arg("output.csv")
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::StanSummary, e.into()))?;

        if !output.status.success() {
            return Err(Self::Error::new(ErrorKind::StanSummary, output.into()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if let Some(line) = stdout.lines().find(|l| l.starts_with("theta")) {
            let mut iter = line.split_whitespace();
            let f = |x: &str| x.parse::<f64>().ok();
            let mean = iter.nth(1).and_then(f);
            let stddev = iter.nth(1).and_then(f);
            match (mean, stddev) {
                (Some(mean), Some(stddev)) if mean - stddev < 0.2 && 0.2 < mean + stddev => (),
                _ => return Err(Self::Error::new(ErrorKind::Bernoulli, output.into())),
            }
        } else {
            return Err(Self::Error::new(ErrorKind::Bernoulli, output.into()));
        }

        let output = Command::new(&inner.diagnose)
            .current_dir(&inner.root)
            .arg("output.csv")
            .output()
            .map_err(|e| Self::Error::new(ErrorKind::Diagnose, e.into()))?;
        if !output.status.success() {
            return Err(Self::Error::new(ErrorKind::Diagnose, output.into()));
        }
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if !stdout.contains("Processing complete, no problems detected") {
            return Err(Self::Error::new(ErrorKind::Bernoulli, output.into()));
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
        let exec = prog.path.with_extension(OS_EXE_EXT);

        // Compilation has the potential to touch all of the files in
        // the CmdStan directory.
        let guard = self.inner.write().unwrap();

        // We need to detect whether the diagnose and stansummary utilities
        // will be deleted. If combined with invalid unicode, it will be difficult
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
        // compilation can fail.
        let output = Command::new(MAKE)
            .current_dir(&guard.root)
            .args(args)
            .arg(&exec)
            .output()
            .map_err(|e| Error::new(ErrorKind::Compilation, e.into()))?;

        if !output.status.success() {
            return Err(Error::new(ErrorKind::Compilation, output.into()));
        }

        // If `clean-all` occurred, then we need to re-build the utilities
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
        let op = |e: io::Error| Self::Error::new(ErrorKind::Executable, e.into());
        // The executable must exist at the time of construction, not be
        // a hypothetical path at which an executable might later appear.
        let exec = fs::canonicalize(path).map_err(op)?;
        let output = try_exec(&exec).map_err(op)?;
        Error::appears_ok(ErrorKind::Executable, output)?;

        Ok(Self { exec })
    }
}
