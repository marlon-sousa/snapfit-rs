// Facade: snapfit's public API, the re-exports of the `Container` trait and of the `container` macro.

#![doc = include_str!("../README.md")]
#![deny(warnings)]

extern crate self as snapfit;

mod container;

#[cfg(test)]
mod tests;

pub use container::Container;
pub use snapfit_macros::container;
