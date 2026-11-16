#![deny(warnings)]

extern crate self as snapfit;

pub mod container;

#[cfg(test)]
mod tests;

pub use container::Container;
pub use snapfit_macros::{Container, container};
