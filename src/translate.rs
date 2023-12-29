use std::ffi::OsString;

pub use translate_derive::*;

/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait Translate: private::Sealed {
    /// Write `self` to `s` as a statement in command line language.
    fn write_stmt(&self, s: &mut OsString);
    /// Write `self` to `s` as a tree, with offset (from left) of `n`.
    fn write_tree_offset(&self, n: usize, s: &mut OsString);
    /// Translate `self` to command line arguments and append to `v`.
    fn append_args(&self, v: &mut Vec<OsString>);

    /// Translate `self` to a statement in command line language.
    fn to_stmt(&self) -> OsString {
        let mut s = OsString::new();
        self.write_stmt(&mut s);
        s
    }
    /// Translate `self` to a tree (pretty but verbose equivalent to a statement).
    fn to_tree(&self) -> OsString {
        let mut s = OsString::new();
        self.write_tree(&mut s);
        s
    }
    /// Write `self` to `s` as a tree.
    fn write_tree(&self, s: &mut OsString) {
        self.write_tree_offset(0, s);
    }
    /// Translate `self` to command line arguments.
    fn to_args(&self) -> Vec<OsString> {
        let mut v = Vec::new();
        self.append_args(&mut v);
        v
    }
}

// public within the crate to allow `impl crate::translate::private::Sealed for ...`
// to be included as part of the `#[derive(Translate)]`.
pub(crate) mod private {
    pub trait Sealed {}
}
