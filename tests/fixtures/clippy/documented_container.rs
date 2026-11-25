//! A public library whose crate warns on every public item without a doc,
//! holding a container, a fallible one and a dispatched type, none of whose
//! generated items may be reported.
#![warn(missing_docs)]

use std::sync::Arc;
use snapfit::container;

/// Where orders are read from.
#[snapfit::static_dispatch]
pub trait Database {
    /// The source an order is read from.
    fn source(&self) -> &'static str;
}

/// The database of production.
#[derive(Default)]
pub struct MySql;

impl Database for MySql {
    fn source(&self) -> &'static str {
        "mysql"
    }
}

/// The settings of the shop.
#[derive(Default, Clone)]
pub struct Config;

/// Settings that may fail to load.
#[derive(Clone)]
pub struct Loaded;

impl Loaded {
    /// Loads the settings.
    pub fn load() -> Result<Self, std::io::Error> {
        Ok(Self)
    }
}

/// The shop's container.
#[container]
#[static_dispatch_for(Database as Db)]
pub struct Shop {
    #[singleton(build = default)]
    config: Config,

    #[singleton(implementation = MySql, build = default)]
    db: Arc<Db>,
}

/// A container one of whose fields can fail.
#[container]
pub struct Loader {
    #[singleton(fallible = true, build = constructor(load))]
    loaded: Loaded,
}
