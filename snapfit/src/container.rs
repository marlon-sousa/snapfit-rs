/// Implemented by every synchronous `#[container]` struct.
///
/// The `#[container]` attribute writes the implementation: a container is
/// built with `new()`, and each of its fields is then read with its
/// generated getter, `get_<field>()`.
/// [The Snapfit Book](https://marlon-sousa.github.io/snapfit-rs/book/) shows how.
pub trait Container: Sized {
    /// Builds every singleton in dependency order.
    fn new() -> Self;
}
