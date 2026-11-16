/// Implemented by every synchronous `#[container]` and `#[derive(Container)]` struct.
pub trait Container: Sized {
    /// Builds every singleton in dependency order.
    fn new() -> Self;
}
