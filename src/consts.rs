//! Various constants

// Extension must be included when used to compile models (on windows).
#[cfg(unix)]
pub(crate) const OS_EXE_EXT: &str = "";
#[cfg(windows)]
pub(crate) const OS_EXE_EXT: &str = "exe";

// The .exe is optional for windows when running via `std::process::Command`,
// but, for the sake of consistency in error messages (and paranoia about windows)
// it costs nothing to configure these at compile time.
#[cfg(unix)]
pub(crate) const STANC: &str = "stanc";
#[cfg(windows)]
pub(crate) const STANC: &str = "stanc.exe";

#[cfg(unix)]
pub(crate) const STANSUMMARY: &str = "stansummary";
#[cfg(windows)]
pub(crate) const STANSUMMARY: &str = "stansummary.exe";

#[cfg(unix)]
pub(crate) const DIAGNOSE: &str = "diagnose";
#[cfg(windows)]
pub(crate) const DIAGNOSE: &str = "diagnose.exe";

// This is utterly necessary
#[cfg(unix)]
pub(crate) const MAKE: &str = "make";
#[cfg(windows)]
pub(crate) const MAKE: &str = "mingw32-make";

// Tokens for make
#[cfg(unix)]
pub(crate) const MAKE_BERNOULLI: &str = "examples/bernoulli/bernoulli";
#[cfg(windows)]
pub(crate) const MAKE_BERNOULLI: &str = "examples/bernoulli/bernoulli.exe";

#[cfg(unix)]
pub(crate) const MAKE_STANC: &str = "bin/stanc";
#[cfg(windows)]
pub(crate) const MAKE_STANC: &str = "bin/stanc.exe";

#[cfg(unix)]
pub(crate) const MAKE_STANSUMMARY: &str = "bin/stansummary";
#[cfg(windows)]
pub(crate) const MAKE_STANSUMMARY: &str = "bin/stansummary.exe";

#[cfg(unix)]
pub(crate) const MAKE_DIAGNOSE: &str = "bin/diagnose";
#[cfg(windows)]
pub(crate) const MAKE_DIAGNOSE: &str = "bin/diagnose.exe";

pub(crate) const HELP: &str = "help";
pub(crate) const DHELP: &str = "--help";

// Common defaults for BFGS/l-BFGS
pub(crate) const INIT_ALPHA: f64 = 0.001;
pub(crate) const TOL_OBJ: f64 = 1e-12;
pub(crate) const TOL_REL_OBJ: f64 = 10_000.0;
pub(crate) const TOL_GRAD: f64 = 1e-8;
pub(crate) const TOL_REL_GRAD: f64 = 10_000_000.0;
pub(crate) const TOL_PARAM: f64 = 1e-8;
pub(crate) const HISTORY_SIZE: i32 = 5;


// Used in multiple places
pub(crate) const NEG1_I32: i32 = -1;
pub(crate) const NEG1_I64: i64 = -1;
pub(crate) const OUTPUT_FILE: &str = "output.csv";
pub(crate) const PROFILE_FILE: &str = "profile.csv";
