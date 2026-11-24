//! What snapfit's container would generate for a statically dispatched
//! field, written by hand: an alias naming the enum, and one `From` per
//! implementation, so that a factory writes `.into()`. This module imports
//! neither `Order` nor `QueryError`.

use std::sync::Arc;

use snapfit::container;

use crate::db::Database;
use crate::impls::{Mock, Postgres};

/// GENERATED in the spike's design: `pub type ShopDb = ::snapfit::OneOf2<Postgres, Mock>;`
pub type ShopDb = oneof::OneOf2<Postgres, Mock>;

/// GENERATED: one `From` per implementation.
impl From<Postgres> for ShopDb {
    fn from(value: Postgres) -> Self { oneof::OneOf2::First(value) }
}
impl From<Mock> for ShopDb {
    fn from(value: Mock) -> Self { oneof::OneOf2::Second(value) }
}

#[derive(Default)]
pub struct Config {
    pub use_mock: bool,
}

impl Config {
    pub fn new() -> Self { Self { use_mock: std::env::var("USE_MOCK").is_ok() } }
}

/// A service generic over the database: no pointer, no vtable, and it does
/// not name the container's alias.
pub struct Orders<D: Database> {
    pub db: D,
}

impl<D: Database> Orders<D> {
    pub fn new(db: D) -> Self { Self { db } }
    pub fn describe(&self) -> String { format!("orders on {}", self.db.name()) }
}

/// A service that names the alias, coupled to this container.
pub struct Reports {
    pub db: ShopDb,
}

impl Reports {
    pub fn new(db: ShopDb) -> Self { Self { db } }
}

#[container]
pub struct Shop {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config], build = factory)]
    db: ShopDb,

    #[singleton(deps = [db])]
    orders: Arc<Orders<ShopDb>>,

    #[singleton(deps = [db])]
    reports: Arc<Reports>,
}

impl ShopFactories for Shop {
    fn build_db(inputs: ShopDbInputs) -> ShopDb {
        if inputs.config.use_mock { Mock.into() } else { Postgres { url: "pg://shop".into() }.into() }
    }
}
