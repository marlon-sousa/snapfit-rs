//! A dispatched trait whose methods a `cfg_attr` removes or keeps.

use snapfit::__private::{Either, Only};

pub type Lamps = Either<Bulb, Only<Candle>>;

#[snapfit::static_dispatch]
pub trait Lamp {
    /// Its condition holds, so the `cfg` it writes removes the method.
    #[cfg_attr(all(), cfg(any()))]
    fn flicker(&self) -> u32;

    /// Its condition fails, so the method stays.
    #[cfg_attr(any(), cfg(any()))]
    fn shine(&self) -> &'static str;
}

#[derive(Debug)]
pub struct Bulb;

#[derive(Debug)]
pub struct Candle;

impl Lamp for Bulb {
    fn shine(&self) -> &'static str {
        "bulb"
    }
}

impl Lamp for Candle {
    fn shine(&self) -> &'static str {
        "candle"
    }
}
