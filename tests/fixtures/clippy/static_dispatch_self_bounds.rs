//! A dispatched trait whose supertrait names `Self::Item` and whose `where`
//! clause bounds `Self` itself, dispatched by a container to two
//! implementations, for which neither rustc nor clippy may report anything.

use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Holder<T> {
    fn hold(&self) -> Option<T>;
}

#[snapfit::static_dispatch]
pub trait Named {
    fn name(&self) -> String;
}

#[snapfit::static_dispatch]
pub trait Store: Named + Holder<Self::Item>
where
    Self: Clone,
{
    type Item;
    fn count(&self) -> u8;
}

#[derive(Clone, Default)]
pub struct Shelf;

#[derive(Clone, Default)]
pub struct Drawer;

impl Holder<u8> for Shelf {
    fn hold(&self) -> Option<u8> {
        Some(1)
    }
}

impl Holder<u8> for Drawer {
    fn hold(&self) -> Option<u8> {
        Some(2)
    }
}

impl Named for Shelf {
    fn name(&self) -> String {
        "shelf".to_owned()
    }
}

impl Named for Drawer {
    fn name(&self) -> String {
        "drawer".to_owned()
    }
}

impl Store for Shelf {
    type Item = u8;
    fn count(&self) -> u8 {
        3
    }
}

impl Store for Drawer {
    type Item = u8;
    fn count(&self) -> u8 {
        4
    }
}

/// A room whose stores hold u8.
#[container]
#[static_dispatch_for(Store as Stores)]
pub struct Room {
    #[singleton(implementation = Shelf, build = default)]
    shelf: Arc<Stores>,

    #[singleton(implementation = Drawer, build = default)]
    drawer: Stores,
}

/// What the room's stores count and hold, and the name of the first.
pub fn tally() -> (u8, String) {
    let room = Room::new();
    let held = room.get_drawer().hold().unwrap_or_default();
    (room.get_shelf().count() + held, room.get_shelf().name())
}
