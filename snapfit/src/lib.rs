#![deny(warnings)]

extern crate self as snapfit;

pub mod container;

#[cfg(test)]
mod tests;

pub use container::Container;
pub use snapfit_macros::container;

/// Compiles and runs the README examples as doctests, so the front page
/// cannot drift from the real API.
#[doc = include_str!("../../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
