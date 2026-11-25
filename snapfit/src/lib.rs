// Facade: snapfit's public API, the re-exports of the `Container` and `FallibleContainer` traits, of `BuildError` and of the `container` and `static_dispatch` macros, and the hidden `__private` module of what generated code calls.

#![doc = include_str!("../README.md")]
#![deny(warnings)]

extern crate self as snapfit;

mod build_error;
mod build_result;
mod container;
mod dispatch;
mod fallible_container;

#[cfg(test)]
mod tests;

pub use build_error::BuildError;
pub use container::Container;
pub use fallible_container::FallibleContainer;
pub use snapfit_macros::{container, static_dispatch};

/// What the code snapfit's macros generate names, and nothing else should:
/// user code reaches the dispatch types through the alias a container
/// declares for each.
#[doc(hidden)]
pub mod __private {
    pub use crate::build_result::BuildResult;
    pub use crate::dispatch::{
        Either, Here, Holds, NotOnTheList, Only, SameType, There, same_type,
    };
}
