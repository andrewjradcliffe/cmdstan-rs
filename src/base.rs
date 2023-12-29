use crate::argument_tree::ArgumentTree;
use crate::constants::*;
use crate::error::*;
use crate::stansummary::StanSummaryOptions;
use crate::translate::Translate;
use std::{
    convert::TryFrom,
    env,
    ffi::{OsStr, OsString},
    fs::{self, File},
    hash::Hash,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
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

/// Try to determine if the binary file exists and is executable by attempting
/// to call it with a single argument (which should be `"--help"` or `"help"`
/// depending on the binary).
fn try_help<S: AsRef<OsStr>>(path: S, help: &'static str) -> Result<process::Output, io::Error> {
    Command::new(path).arg(help).output()
}

/// Holds an absolute path to a Stan program. Invariants established
/// at the time of construction cannot be guaranteed to be true at all times,
/// as it is always possible to modify or delete the underlying file.
#[derive(Debug, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct StanProgram {
    path: PathBuf,
}

impl TryFrom<&Path> for StanProgram {
    type Error = Error;
    /// Try to create an instance from the given `path`. At the time of construction,
    /// the path must exist on disk and be pointing at file whose extension is `"stan"`.
    /// The path will be canonicalized at the point of construction, if the
    /// aforementioned conditions are met.
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
        // at the point of creation. If a user understands this, then they can plan
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
#[derive(Debug, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
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
    ($F:ident, $target:ident, $kind:expr, $field:ident, $help:ident) => {
        fn $F(&self) -> Result<(), Error> {
            let output = self
                .try_ensure_help(&self.$field, $target, $help)
                .map_err(|e| Error::new($kind, e.into()))?;
            Error::appears_ok($kind, output)
        }
    };
}

/// Operations to be called only from within a `CmdStan` instance where
/// one has write access to the `inner` field.
/// Alternatively, during `CmdStanInner::try_from` or `CmdStan::try_from`
impl CmdStanInner {
    fn try_ensure(&self, bin: &Path, target: &'static str) -> io::Result<process::Output> {
        match try_exec(bin) {
            Ok(x) => Ok(x),
            Err(_) => {
                self.make(target)?;
                try_exec(bin)
            }
        }
    }
    fn try_ensure_help(
        &self,
        bin: &Path,
        target: &'static str,
        help: &'static str,
    ) -> io::Result<process::Output> {
        match try_help(bin, help) {
            Ok(x) => Ok(x),
            Err(_) => {
                self.make(target)?;
                try_help(bin, help)
            }
        }
    }
    impl_try_ensure!(try_ensure_stanc, MAKE_STANC, ErrorKind::StanC, stanc, DHELP);
    impl_try_ensure!(
        try_ensure_stansummary,
        MAKE_STANSUMMARY,
        ErrorKind::StanSummary,
        stansummary,
        DHELP
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

/// Construct which represents a unique directory on disk at which
/// CmdStan is installed. This type uses `Arc<RwLock<_>>` internally
/// in order to synchronize concurrent method calls which have the potential
/// cause data races on the filesystem.
///
/// Synchronization is necessary to prevent inconsistent (or simply failing)
/// results which would otherwise arise due to concurrent compilation of Stan programs
/// with different options. If this were not performed internally, the user
/// would be forced to take on this burden; it is far easier (and more efficient)
/// to implement this as part of the library design, thereby enabling the user
/// to treat this construct as an opaque call handler across arbitrary threads.
///
/// Consequently, the appropriate way to use `CmdStan` is to construct
/// it once per unique directory, using [`CmdStan::try_from`], which performs
/// a number of invariant validation steps, and to `clone` when one needs
/// to send a copy to other threads. `CmdStan` uses `Arc` internally,  hence,
/// `clones`  are cheap.
///
/// That said, it is still possible to violate the invariants by constructing
/// multiple instances by calling `CmdStan::try_from` on the same input path,
/// then writing a program which causes concurrent compilation. Given the fundamental
/// basis on the filesystem, there is nothing a library writer can do to stop
/// users from such engaging in such nonsense. Naturally, informative error
/// messages would be returned if the unsynchronized concurrent writes within
/// the installation (or target) directory result in an error, but an error
/// is not guaranteed.
///
/// Lastly, multiple `CmdStan`s constructed by calling `CmdStan::try_from` on
/// distinct directories are entirely permissible. However, one must note
/// that `CmdStan::compile`, called concurrently on the same `StanProgram`
/// but with `CmdStan` instances respective to distinct directories
/// would result in a race to compile the target binary. Again, this is
/// is a case where the library writer cannot protect the user from the racy
/// nature of the filesystem.
#[derive(Debug, Clone)]
pub struct CmdStan {
    inner: Arc<RwLock<CmdStanInner>>,
}

impl TryFrom<&Path> for CmdStan {
    type Error = Error;
    /// Try to create an instance from the given `path`. At the time of construction,
    /// the following invariants are established:
    /// - the directory is a CmdStan installation
    /// - `stanc`, `stansummary`, and `diagnose` binaries are built
    /// - the Bernoulli example can be compiled
    /// - the Bernoulli executable, when run, produces satisfactory results
    ///
    /// Taken together, these may be an expensive set of operations, depending
    /// on the state of the directory.
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
        Self::Error::appears_ok(ErrorKind::Bernoulli, output)?;

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

/** Operations which acquire write access

- `compile` : has the potential to modify all files in the root directory of `self`.
- `stanc` : may write to a `StanProgram`'s (generated) C++ program file; such a write
would race with other such `stanc` calls.
*/
impl CmdStan {
    pub fn compile<I, S>(&self, program: &StanProgram, args: I) -> Result<CmdStanModel, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let exec = program.path.with_extension(OS_EXE_EXT);

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

    pub fn stanc<I, S>(&self, program: &StanProgram, args: I) -> Result<process::Output, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let guard = self.inner.write().unwrap();
        Command::new(&guard.stanc)
            .current_dir(&guard.root)
            .args(args)
            .arg(&program.path)
            .output()
            .map_err(|e| Error::new(ErrorKind::StanC, e.into()))
    }
}

/** Operations which acquire read access

- `diagnose` : does not modify any files in the root directory of `self`
- `stansummary` : does not modify any files in the root directory of `self`
*/
impl CmdStan {
    pub fn diagnose(&self, output: &CmdStanOutput) -> Result<process::Output, Error> {
        let guard = self.inner.read().unwrap();
        Command::new(&guard.diagnose)
            .args(output.output_files())
            .output()
            .map_err(|e| Error::new(ErrorKind::Diagnose, e.into()))
    }
    pub fn stansummary<T>(&self, output: &CmdStanOutput, opts: T) -> Result<process::Output, Error>
    where
        T: Into<Option<StanSummaryOptions>>,
    {
        let guard = self.inner.read().unwrap();
        let mut cmd = Command::new(&guard.stansummary);
        cmd.args(output.output_files());
        if let Some(opts) = opts.into() {
            cmd.args(opts.command_fragment());
        }
        cmd.output()
            .map_err(|e| Error::new(ErrorKind::StanSummary, e.into()))
    }
}

/// Holds an absolute path to a compiled executable. Invariants established
/// at the time of construction cannot be guaranteed to be true at all times,
/// as it is always possible to modify or delete the underlying file.
#[derive(Debug, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct CmdStanModel {
    exec: PathBuf,
}

impl TryFrom<&Path> for CmdStanModel {
    type Error = Error;
    /// Try to create an instance from the given `path`. At the time of construction,
    /// the path must exist on disk and be pointing at a compiled executable.
    /// The path will be canonicalized at the point of construction, if the
    /// aforementioned conditions are met.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // The executable must exist at the time of construction, not be
        // a hypothetical path at which an executable might later appear.
        let exec = fs::canonicalize(path).map_err(Self::error_op)?;
        let output = try_help(&exec, HELP).map_err(Self::error_op)?;
        Self::Error::appears_ok(ErrorKind::Executable, output)?;

        Ok(Self { exec })
    }
}
// Worthwhile? not certain.
// impl TryFrom<StanProgram> for CmdStanModel {
//     type Error = Error;
//     fn try_from(program: StanProgram) -> Result<Self, Self::Error> {
//         let mut exec = program.path;
//         exec.set_extension(OS_EXE_EXT);
//         let output = try_help(&exec, HELP).map_err(Self::error_op)?;
//         Self::Error::appears_ok(ErrorKind::Executable, output)?;
//         Ok(Self { exec })
//     }
// }

use std::collections::HashMap;
impl CmdStanModel {
    /// Associated function which provides error of default kind for `CmdStanModel`
    /// and converts the error representation (be it IO or failed process).
    fn error_op<E: Into<Repr>>(e: E) -> Error {
        Error::new(ErrorKind::Executable, e.into())
    }

    fn info(&self) -> Result<HashMap<String, String>, Error> {
        let output = Command::new(&self.exec)
            .arg("info")
            .output()
            .map_err(Self::error_op)?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            let map: HashMap<String, String> = stdout
                .lines()
                .filter_map(|line| line.split_once('='))
                .map(|(lhs, rhs)| (String::from(lhs.trim()), String::from(rhs.trim())))
                .collect();
            Ok(map)
        } else {
            Err(Self::error_op(output))
        }
    }

    /// Call the compiled model with the arguments contained in `tree`.
    /// Log files, containing the `stdout` and `stderr` of the spawned process,
    /// will be created in the same directory at which the `tree.output.file`
    /// is created; the logs are populated dynamically to facilitate monitoring,
    /// rather than through a single bulk update.
    ///
    /// Successful return requires that the output of the spawned process
    /// have a zero exit status. If the exit status is non-zero,
    /// an appropriate error term will be returned with the `process::Output`
    /// `stdout` and `stderr` read from the respective log files.
    pub fn call(&self, tree: &ArgumentTree) -> Result<CmdStanOutput, Error> {
        let cwd = env::current_dir().map_err(Self::error_op)?;
        let out: &Path = tree.output.file.as_ref();
        // The log name likely needs to be unique, else we risk clobbering
        // someone's precious file of the same name.
        let mut stdout = if out.is_relative() {
            cwd.join(out)
        } else {
            out.to_path_buf()
        };
        stdout.set_extension("");
        let mut stderr = stdout.clone();
        stdout.as_mut_os_string().push("_stdout_log.txt");
        stderr.as_mut_os_string().push("_stderr_log.txt");

        // Pipe both stdout and stderr to separate log files
        let out = File::create(&stdout).map_err(Self::error_op)?;
        let err = File::create(&stderr).map_err(Self::error_op)?;
        let mut output = Command::new(&self.exec)
            .args(tree.to_args())
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err)
            .output()
            .map_err(Self::error_op)?;
        if output.status.success() {
            Ok(CmdStanOutput {
                stdout_path: stdout,
                stderr_path: stderr,
                cwd_at_call: cwd,
                output,
                argument_tree: tree.clone(),
            })
        } else {
            // However, we need cook up an equivalent `process::Output`
            // by reading the bytes we dumped to file.
            // Leaving the log files on disk is likely desirable,
            // in the event that something catastrophic happens...
            // or the user just ignores the error thrown by this call.
            let mut out = File::open(&stdout).map_err(Self::error_op)?;
            let mut err = File::open(&stderr).map_err(Self::error_op)?;
            out.read_to_end(&mut output.stdout)
                .map_err(Self::error_op)?;
            err.read_to_end(&mut output.stdout)
                .map_err(Self::error_op)?;
            Err(Self::error_op(output))
        }
    }
}

// #[allow(non_snake_case)]
// pub struct ModelInfo {
//     pub stan_version_major: u32,
//     pub stan_version_minor: u32,
//     pub stan_version_patch: u32,
//     pub STAN_THREADS: bool,
//     pub STAN_MPI: bool,
//     pub STAN_OPENCL: bool,
//     pub STAN_NO_RANGE_CHECKS: bool,
//     pub STAN_CPP_OPTIMS: bool,
// }

/// A snapshot produced by performing `CmdStanModel::call`.
/// This is a self-contained record, the contents of which include:
///
/// - the console output (exit status, stdout and stderr),
/// - the argument tree with the call was made
/// - the current working directory of the process at the time the call was made
pub struct CmdStanOutput {
    /// Enables methods such as `output_files`, `diagnostic_files`,
    /// etc. to return absolute paths by introspection of the
    /// `ArgumentTree` and `cwd_at_call`.  In essence, if the
    /// output/diagnostic/profile file is relative, then it should be
    /// pushed onto `cwd_at_call`.
    cwd_at_call: PathBuf,
    output: process::Output,
    argument_tree: ArgumentTree,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}
impl CmdStanOutput {
    /// Convert files to absolute paths. If the file is already
    /// absolute, this is a no-op (simple move into `PathBuf`);
    /// otherwise the current working directory at the time this
    /// `CmdStanOutput` instance was created serves as the prefix onto
    /// which the relative path will be joined.
    fn files<F>(&self, f: F) -> Vec<PathBuf>
    where
        F: Fn(&ArgumentTree) -> Vec<OsString>,
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
            .filter(|path| path.is_file())
            .collect()
    }
    /// Return the output files associated with the call.
    pub fn output_files(&self) -> Vec<PathBuf> {
        self.files(|tree| tree.output_files())
    }
    /// Return the diagnostic files associated with the call.
    pub fn diagnostic_files(&self) -> Vec<PathBuf> {
        self.files(|tree| tree.diagnostic_files())
    }
    /// Return the profile files associated with the call.
    pub fn profile_files(&self) -> Vec<PathBuf> {
        self.files(|tree| tree.profile_files())
    }

    /// Return a reference to the log file which contains the console output.
    pub fn stdout_file(&self) -> &Path {
        &self.stdout_path
    }
    /// Return a reference to the log file which contains the console error.
    pub fn stderr_file(&self) -> &Path {
        &self.stderr_path
    }

    /// Return a reference to console output of the call.
    pub fn output(&self) -> &process::Output {
        &self.output
    }

    /// Return a reference to the current working directory at the
    /// time of the call.
    pub fn cwd_at_call(&self) -> &Path {
        &self.cwd_at_call
    }

    /// Return a reference to the argument tree with which the call
    /// was made.
    pub fn argument_tree(&self) -> &ArgumentTree {
        &self.argument_tree
    }
}
