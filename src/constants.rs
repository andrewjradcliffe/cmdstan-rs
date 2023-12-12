//! Various constants

// Extension must be included when used to compile models (on windows).
#[cfg(unix)]
pub(crate) static OS_EXE_EXT: &str = "";
#[cfg(windows)]
pub(crate) static OS_EXE_EXT: &str = "exe";

// The .exe is optional for windows when running via `std::process::Command`,
// but, for the sake of consistency in error messages (and paranoia about windows)
// it costs nothing to configure these at compile time.
#[cfg(unix)]
pub(crate) static STANC: &str = "stanc";
#[cfg(windows)]
pub(crate) static STANC: &str = "stanc.exe";

#[cfg(unix)]
pub(crate) static STANSUMMARY: &str = "stansummary";
#[cfg(windows)]
pub(crate) static STANSUMMARY: &str = "stansummary.exe";

#[cfg(unix)]
pub(crate) static DIAGNOSE: &str = "diagnose";
#[cfg(windows)]
pub(crate) static DIAGNOSE: &str = "diagnose.exe";

// This is utterly necessary
#[cfg(unix)]
pub(crate) static MAKE: &str = "make";
#[cfg(windows)]
pub(crate) static MAKE: &str = "mingw32-make";

// Tokens for make
#[cfg(unix)]
pub(crate) static MAKE_BERNOULLI: &str = "examples/bernoulli/bernoulli";
#[cfg(windows)]
pub(crate) static MAKE_BERNOULLI: &str = "examples/bernoulli/bernoulli.exe";

#[cfg(unix)]
pub(crate) static MAKE_STANC: &str = "bin/stanc";
#[cfg(windows)]
pub(crate) static MAKE_STANC: &str = "bin/stanc.exe";

#[cfg(unix)]
pub(crate) static MAKE_STANSUMMARY: &str = "bin/stansummary";
#[cfg(windows)]
pub(crate) static MAKE_STANSUMMARY: &str = "bin/stansummary.exe";

#[cfg(unix)]
pub(crate) static MAKE_DIAGNOSE: &str = "bin/diagnose";
#[cfg(windows)]
pub(crate) static MAKE_DIAGNOSE: &str = "bin/diagnose.exe";

pub(crate) static HELP: &str = "help";
pub(crate) static DHELP: &str = "--help";
