#![deny(warnings)]

extern crate self as snapfit;

pub mod container;
pub mod error;
pub mod introspection;

#[cfg(test)]
mod tests;

pub use container::{AsyncContainer, Container};
pub use error::SnapfitError;
pub type Error = SnapfitError;
pub use introspection::{EdgeDescriptor, GraphDescriptor, NodeDescriptor};
pub use snapfit_macros::{Container, container};
