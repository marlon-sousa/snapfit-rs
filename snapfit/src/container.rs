/// Implemented by every synchronous `#[container]` struct.
pub trait Container: Sized {
    /// Builds every singleton in dependency order.
    fn new() -> Self;
}
