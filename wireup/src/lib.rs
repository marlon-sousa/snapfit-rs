#![deny(warnings)]

extern crate self as wireup;

pub mod container;
pub mod error;
pub mod introspection;

#[cfg(test)]
mod tests;

pub use container::{AsyncContainer, Container};
pub use error::WireupError;
pub type Error = WireupError;
pub use introspection::{EdgeDescriptor, GraphDescriptor, NodeDescriptor};
pub use wireup_macros::{Container, container};
