//! Desks whose trait-object readers are built from their implementation in
//! every way a field can be built: by a named constructor, by `Default`, by
//! a builder, by the container's `leaf_build`, and by a constructor that can fail.

use std::sync::Arc;

use snapfit::container;

use super::readers::Reader;

/// A reader that says how it was built.
#[derive(typed_builder::TypedBuilder)]
pub struct ShelfReader {
    #[builder(default = "shelf built by its builder")]
    source: &'static str,
}

impl ShelfReader {
    pub fn open() -> Self {
        Self {
            source: "shelf opened",
        }
    }

    pub fn checked() -> Result<Self, String> {
        Ok(Self {
            source: "shelf checked",
        })
    }

    pub fn locked() -> Result<Self, String> {
        Err("the shelf is locked".to_owned())
    }
}

impl Default for ShelfReader {
    fn default() -> Self {
        Self {
            source: "shelf by default",
        }
    }
}

impl Reader for ShelfReader {
    fn source(&self) -> &'static str {
        self.source
    }
}

#[container]
pub struct BuiltDesk {
    #[singleton(implementation = ShelfReader, build = constructor(open))]
    opened: Arc<dyn Reader>,

    #[singleton(implementation = ShelfReader, build = default)]
    defaulted: Arc<dyn Reader>,

    #[singleton(implementation = ShelfReader, build = builder)]
    built: Arc<dyn Reader>,

    #[singleton(implementation = ShelfReader, build = constructor(checked), fallible = true)]
    checked: Arc<dyn Reader>,
}

#[container(leaf_build = default)]
pub struct LeafDesk {
    #[singleton(implementation = ShelfReader)]
    reader: Arc<dyn Reader>,
}

#[container]
pub struct LockedDesk {
    #[singleton(implementation = ShelfReader, build = constructor(locked), fallible = true)]
    reader: Arc<dyn Reader>,
}
