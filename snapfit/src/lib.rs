// Facade: snapfit's public API, the re-exports of the `Container` and `FallibleContainer` traits, of `BuildError` and of the `container` macro, and the hidden `__private` module of what generated code calls.

#![doc = include_str!("../README.md")]
#![deny(warnings)]

extern crate self as snapfit;

mod build_error;
mod build_result;
mod container;
mod fallible_container;

#[cfg(test)]
mod tests;

pub use build_error::BuildError;
pub use container::Container;
pub use fallible_container::FallibleContainer;
pub use snapfit_macros::container;

/// What the code `#[container]` generates calls, and nothing else should.
#[doc(hidden)]
pub mod __private {
    pub use crate::build_result::BuildResult;
}
