//! Dispatched traits whose methods a `cfg_attr` removes or keeps, written
//! by hand and through a `macro_rules!`.

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

    /// Its `cfg` keeps the method, and the `deprecated` it also writes
    /// may not be written on the impls' method.
    #[cfg_attr(all(), cfg(all()), deprecated = "shine instead")]
    fn glow(&self) -> u32;
}

#[derive(Debug)]
pub struct Bulb;

#[derive(Debug)]
pub struct Candle;

impl Lamp for Bulb {
    fn shine(&self) -> &'static str {
        "bulb"
    }

    fn glow(&self) -> u32 {
        1
    }
}

impl Lamp for Candle {
    fn shine(&self) -> &'static str {
        "candle"
    }

    fn glow(&self) -> u32 {
        2
    }
}

/// Passes the condition and the `cfg` as `$m:meta`, which arrive wrapped
/// in groups of their own.
macro_rules! torch {
    ($condition:meta, $removed:meta) => {
        #[snapfit::static_dispatch]
        pub trait Torch {
            #[cfg_attr($condition, $removed)]
            fn flicker(&self) -> u32;

            fn beam(&self) -> &'static str;
        }
    };
}

torch!(all(), cfg(any()));

pub type Torches = Only<Bulb>;

impl Torch for Bulb {
    fn beam(&self) -> &'static str {
        "bulb beam"
    }
}
