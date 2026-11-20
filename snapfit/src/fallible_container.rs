//! Entity: the trait every synchronous container with a fallible field implements.

use crate::BuildError;

/// Implemented by every synchronous `#[container]` struct that has a field
/// marked `fallible = true`, in place of [`Container`](crate::Container).
///
/// The `#[container]` attribute writes the implementation: such a container
/// is built with `try_new()`, which stops at the first field that fails.
pub trait FallibleContainer: Sized {
    /// Builds every singleton in dependency order, or returns the error of
    /// the first one that fails, after dropping the ones already built.
    fn try_new() -> Result<Self, BuildError>;
}
