//! A container that dispatches a trait with two type arguments, holding its
//! implementations in an `Arc` and as a value.

use std::sync::Arc;

use snapfit::container;

#[snapfit::static_dispatch]
pub trait Convert<A, B> {
    fn convert(&self, value: A) -> B;
}

#[derive(Debug, Clone, Default)]
pub struct Doubler;

impl Doubler {
    pub fn new() -> Self {
        Self
    }
}

impl Convert<u8, u16> for Doubler {
    fn convert(&self, value: u8) -> u16 {
        u16::from(value) * 2
    }
}

#[derive(Debug, Clone, Default)]
pub struct Tripler;

impl Tripler {
    pub fn new() -> Self {
        Self
    }
}

impl Convert<u8, u16> for Tripler {
    fn convert(&self, value: u8) -> u16 {
        u16::from(value) * 3
    }
}

#[container]
#[static_dispatch_for(Convert<u8, u16> as Converters)]
pub struct Conversions {
    #[singleton(implementation = Doubler)]
    shared: Arc<Converters>,

    #[singleton(implementation = Tripler)]
    owned: Converters,
}
