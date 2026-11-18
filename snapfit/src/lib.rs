#![doc = include_str!("../README.md")]
#![deny(warnings)]

extern crate self as snapfit;

mod container;

#[cfg(test)]
mod tests;

pub use container::Container;
pub use snapfit_macros::container;
