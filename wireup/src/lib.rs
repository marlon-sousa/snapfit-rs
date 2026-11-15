#![deny(warnings)]

pub mod container;
pub mod error;
pub mod introspection;

pub use container::{AsyncContainer, Container};
pub use error::WireupError;
pub type Error = WireupError;
pub use introspection::{GraphDescriptor, NodeDescriptor};
