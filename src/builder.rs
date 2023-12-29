pub use builder_derive::*;

/// Trait for deriving builder methods on `struct`s and `enum`s.
///
/// For `struct`s, deriving this trait automatically derives `Default`.
/// For `enum`s, a manual implementation of `Default` is required.
pub(crate) trait Builder {}
