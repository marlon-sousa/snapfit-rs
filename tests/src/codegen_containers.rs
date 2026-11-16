//! Containers that once broke snapfit's code generation. Each one is a
//! regression fixture for `features/stable/generated_code.feature`.

use std::sync::Arc;

use snapfit::container;

use crate::services::{Config, Database};

/// Plain, fully qualified and raw-identifier fields: the generator must reuse
/// the types exactly as written instead of re-parsing them from strings.
#[container]
pub struct TypedFieldsContainer {
    #[singleton]
    tags: Vec<String>,

    #[singleton]
    config: ::std::sync::Arc<Config>,

    #[singleton(deps = [config])]
    db: std::sync::Arc<crate::services::Database>,

    #[singleton]
    r#type: std::collections::BTreeMap<String, u32>,
}

/// Derives written before `#[container]`.
#[derive(Clone, Debug)]
#[container]
pub struct DerivedBeforeContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config])]
    db: Arc<Database>,
}

/// Derives written after `#[container]`.
#[container]
#[derive(Clone, Debug)]
pub struct DerivedAfterContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config])]
    db: Arc<Database>,
}

/// What the derive scenarios check, whichever attribute order was used.
pub trait DerivedContainer {
    fn db(&self) -> Arc<Database>;
    fn db_of_clone(&self) -> Arc<Database>;
    fn debug_text(&self) -> String;
}

impl DerivedContainer for DerivedBeforeContainer {
    fn db(&self) -> Arc<Database> {
        self.get_db()
    }

    fn db_of_clone(&self) -> Arc<Database> {
        self.clone().get_db()
    }

    fn debug_text(&self) -> String {
        format!("{self:?}")
    }
}

impl DerivedContainer for DerivedAfterContainer {
    fn db(&self) -> Arc<Database> {
        self.get_db()
    }

    fn db_of_clone(&self) -> Arc<Database> {
        self.clone().get_db()
    }

    fn debug_text(&self) -> String {
        format!("{self:?}")
    }
}

/// A non-snake-case field named like a unit struct in scope: the locals the
/// generated `new()` binds must not be read as patterns of that struct.
pub mod unit_struct_named_field {
    #![allow(non_snake_case)]

    use std::sync::Arc;

    use snapfit::container;

    #[derive(Debug)]
    pub struct Marker;

    impl Marker {
        pub fn new() -> Self {
            Self
        }
    }

    #[container]
    pub struct UnitStructNamedFieldContainer {
        #[singleton]
        Marker: Arc<Marker>,
    }
}
