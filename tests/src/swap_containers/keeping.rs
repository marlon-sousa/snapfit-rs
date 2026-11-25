//! Desks whose services keep the reader's trait object inside their own
//! type, behind an `Arc` of its own, and are built by their constructors: one
//! written by hand, and one whose service's type a `macro_rules!` passes as
//! `$t:ty`.

use std::sync::Arc;

use snapfit::container;

use super::readers::{DiskReader, Reader};

/// A service generic over the reader it keeps.
#[derive(Clone)]
pub struct Shelf<R> {
    pub reader: R,
}

impl<R> Shelf<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn stock(reader: R) -> Self {
        Self { reader }
    }
}

#[container]
pub struct KeepingDesk {
    #[singleton(implementation = DiskReader)]
    reader: Arc<dyn Reader>,

    #[singleton(deps = [reader])]
    shelf: Arc<Shelf<Arc<dyn Reader>>>,

    #[singleton(deps = [reader], build = constructor(stock))]
    stocked: Shelf<Arc<dyn Reader>>,
}

/// A desk whose shelf's type a `macro_rules!` passes as `$t:ty`, which wraps
/// it in an invisible group.
macro_rules! typed_keeping_desk {
    ($shelf:ty) => {
        #[container]
        pub struct TypedKeepingDesk {
            #[singleton(implementation = DiskReader)]
            reader: Arc<dyn Reader>,

            #[singleton(deps = [reader])]
            shelf: $shelf,
        }
    };
}

typed_keeping_desk!(Arc<Shelf<Arc<dyn Reader>>>);
