//! A dispatched trait that a `macro_rules!` declares, its return types passed
//! as tokens.

use snapfit::__private::{Either, Only};

macro_rules! dispatched {
    ($name:ident, $output:ty, $copy:ty) => {
        #[snapfit::static_dispatch]
        pub trait $name {
            fn source(&self) -> $output;

            fn copied(&self) -> $copy;
        }
    };
}

dispatched!(Rack, &'static str, Self);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Oak;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pine;

impl Rack for Oak {
    fn source(&self) -> &'static str {
        "oak"
    }

    fn copied(&self) -> Self {
        self.clone()
    }
}

impl Rack for Pine {
    fn source(&self) -> &'static str {
        "pine"
    }

    fn copied(&self) -> Self {
        self.clone()
    }
}

pub type Racks = Either<Oak, Only<Pine>>;
