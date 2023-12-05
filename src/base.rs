use std::{
    convert::TryFrom,
    env,
    ffi::OsStr,
    fmt,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self, Command},
};

/// Try to determine if the file exists by attempting to open it in read-only mode.
fn try_open<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    File::open(path).map(|_| ())
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

#[cfg(unix)]
static MAKE: &str = "make";
#[cfg(windows)]
static MAKE: &str = "mingw32-make";

#[cfg(unix)]
static OS_EXTENSION: &str = "";
#[cfg(windows)]
static OS_EXTENSION: &str = "exe";

#[cfg(unix)]
static BERNOULLI: &str = "examples/bernoulli/bernoulli";
#[cfg(windows)]
static BERNOULLI: &str = "examples/bernoulli/bernoulli.exe";

pub enum CmdStanError {
    Io(io::Error),
    BernoulliExample,
    StanSummary(io::Error),
    Diagnose(io::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CmdStan {
    path: PathBuf,
}

impl TryFrom<&Path> for CmdStan {
    type Error = CmdStanError;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // A reliable way to determine if the directory exists
        // and is accessible
        fs::read_dir(path).map_err(Self::Error::Io)?;

        // Rather than verify individual files, a simple way to
        // verify CmdStan works is to build and run the bernoulli example
        Self::make_internal(path, BERNOULLI).map_err(Self::Error::Io)?;
        try_open(path.join(BERNOULLI)).map_err(Self::Error::Io)?;

        let output = Command::new(BERNOULLI)
            .current_dir(path)
            .arg("sample")
            .arg("data")
            .arg("file=examples/bernoulli/bernoulli.data.R")
            .output()
            .map_err(Self::Error::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if !stdout.contains("Adjust your expectations accordingly!") {
            return Err(Self::Error::BernoulliExample);
        };

        Self::try_ensure_stansummary(path)?;
        Self::try_ensure_diagnose(path)?;

        let output = Command::new("bin/stansummary")
            .current_dir(path)
            .arg("output.csv")
            .output()
            .map_err(Self::Error::StanSummary)?;

        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if let Some(line) = stdout.lines().find(|l| l.starts_with("theta")) {
            let mut iter = line.split_whitespace();
            let f = |x: &str| x.parse::<f64>().ok();
            let mean = iter.nth(1).and_then(f);
            let stddev = iter.nth(1).and_then(f);
            match (mean, stddev) {
                (Some(mean), Some(stddev)) if mean - stddev < 0.2 && 0.2 < mean + stddev => (),
                _ => return Err(Self::Error::BernoulliExample),
            }
        } else {
            return Err(Self::Error::BernoulliExample);
        }

        let output = Command::new("bin/diagnose")
            .current_dir(path)
            .arg("output.csv")
            .output()
            .map_err(Self::Error::Diagnose)?;
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if !stdout.contains("Processing complete, no problems detected") {
            return Err(Self::Error::BernoulliExample);
        }

        fs::remove_file(path.join("output.csv")).map_err(Self::Error::Io)?;

        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

impl CmdStan {
    fn try_ensure_stansummary(path: &Path) -> Result<(), CmdStanError> {
        let mut bin = path.to_path_buf();
        bin.push("bin");
        bin.push("stansummary");
        match try_open(&bin) {
            Ok(_) => Ok(()),
            Err(_) => {
                match Self::make_internal(path, "bin/stansummary") {
                    Ok(_) => (),
                    Err(e) => return Err(CmdStanError::StanSummary(e)),
                };
                try_open(&bin).map_err(CmdStanError::StanSummary)
            }
        }
    }

    fn try_ensure_diagnose(path: &Path) -> Result<(), CmdStanError> {
        let mut bin = path.to_path_buf();
        bin.push("bin");
        bin.push("diagnose");
        match try_open(&bin) {
            Ok(_) => Ok(()),
            Err(_) => {
                match Self::make_internal(path, "bin/diagnose") {
                    Ok(_) => (),
                    Err(e) => return Err(CmdStanError::Diagnose(e)),
                };
                try_open(&bin).map_err(CmdStanError::Diagnose)
            }
        }
    }

    /// Call `make` with the supplied arguments from the root of the
    /// `CmdStan` directory.
    /// the time between the creation of `self` and this call that the
    /// files on disk have not been irreparably changed.
    fn make<I, S>(&self, args: I) -> io::Result<process::Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new(MAKE)
            .current_dir(&self.path)
            .args(args)
            .output()
    }

    fn make_internal<P, S>(path: P, arg: S) -> io::Result<process::Output>
    where
        P: AsRef<Path>,
        S: AsRef<OsStr>,
    {
        Command::new(MAKE).current_dir(path).arg(arg).output()
    }

    pub fn compile<I, S>(
        &self,
        prog: &StanProgram,
        args: I,
    ) -> Result<CmdStanModel, CompilationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut exec = prog.path.clone();
        exec.set_extension(OS_EXTENSION);

        // This is lazy, but, not unreasonable given the myriad ways in which
        // things can fail.
        let mut cmd = Command::new(MAKE);
        cmd.current_dir(&self.path).args(args).arg(&exec);

        let output = cmd.output().map_err(CompilationError::Io)?;
        let output2 = cmd.output().map_err(CompilationError::Io)?;
        // If the first attempt was successful, then make will return
        // a message to the effect of "... is up to date." in standard output.
        let stdout = String::from_utf8_lossy(&output2.stdout[..]);
        if !stdout.trim_end().ends_with("is up to date.") {
            return Err(CompilationError::StanCompiler(output));
        }
        // Then, we subject the binary to the same tests as are required
        // to construct directly from a path.
        CmdStanModel::try_from(exec.as_ref()).map_err(|e| match e {
            ModelError::Io(e) => CompilationError::Io(e),
            ModelError::InvalidExecutable => CompilationError::InvalidExecutable,
        })
    }
}

// pub struct StanSummaryUtil {
//     path: PathBuf,
// }

// impl TryFrom<&CmdStan> for StanSummaryUtil {
//     type Error = CmdStanError;
//     fn try_from(cmdstan: &CmdStan) -> Result<Self, Self::Error> {
//         let mut bin = cmdstan.path.clone();
//         bin.push("bin");
//         bin.push("stansummary");
//         match try_open(&bin) {
//             Ok(_) => Ok(Self { path: bin }),
//             Err(_) => {
//                 match cmdstan.make(&["bin/stansummary"]) {
//                     Ok(_) => (),
//                     Err(e) => return Err(CmdStanError::StanSummary(e)),
//                 };
//                 match try_open(&bin) {
//                     Ok(_) => Ok(Self { path: bin }),
//                     Err(e) => Err(CmdStanError::StanSummary(e)),
//                 }
//             }
//         }
//     }
// }

// pub struct DiagnoseUtil {
//     path: PathBuf,
// }

// impl TryFrom<&CmdStan> for DiagnoseUtil {
//     type Error = CmdStanError;
//     fn try_from(cmdstan: &CmdStan) -> Result<Self, Self::Error> {
//         let mut bin = cmdstan.path.clone();
//         bin.push("bin");
//         bin.push("diagnose");
//         match try_open(&bin) {
//             Ok(_) => Ok(Self { path: bin }),
//             Err(_) => {
//                 match cmdstan.make(&["bin/diagnose"]) {
//                     Ok(_) => (),
//                     Err(e) => return Err(CmdStanError::Diagnose(e)),
//                 };
//                 match try_open(&bin) {
//                     Ok(_) => Ok(Self { path: bin }),
//                     Err(e) => Err(CmdStanError::Diagnose(e)),
//                 }
//             }
//         }
//     }
// }

#[derive(Debug)]
pub enum CompilationError {
    Io(io::Error),
    StanCompiler(process::Output),
    InvalidExecutable,
}

pub enum ModelError {
    Io(io::Error),
    InvalidExecutable,
}

pub struct CmdStanModel {
    exec: PathBuf,
}

impl TryFrom<&Path> for CmdStanModel {
    type Error = ModelError;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        try_open(path).map_err(ModelError::Io)?;

        let output = Command::new(path)
            .arg("help")
            .output()
            .map_err(ModelError::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        if !stdout.contains("Bayesian inference with Markov Chain Monte Carlo") {
            return Err(ModelError::InvalidExecutable);
        }
        Ok(Self {
            exec: path.to_path_buf(),
        })
    }
}
