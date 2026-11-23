#![deny(warnings)]
//! Chapter 10: dependencies passed by name, through a factory.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: domain
pub struct Database {
    role: &'static str,
}

impl Database {
    pub fn primary() -> Self {
        Self { role: "primary-rw" }
    }

    pub fn replica() -> Self {
        Self { role: "replica-ro" }
    }

    pub fn role(&self) -> &'static str {
        self.role
    }
}

pub struct CqrsOrderStore {
    read_db: Arc<Database>,
    write_db: Arc<Database>,
}

impl CqrsOrderStore {
    pub fn new(read_db: Arc<Database>, write_db: Arc<Database>) -> Self {
        Self { read_db, write_db }
    }

    pub fn describe_routing(&self) -> String {
        format!(
            "reads go to {}, writes go to {}",
            self.read_db.role(),
            self.write_db.role()
        )
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton(build = constructor(replica))]
    read_db: Arc<Database>,

    #[singleton(build = constructor(primary))]
    write_db: Arc<Database>,

    #[singleton(deps = [write_db, read_db], build = factory)]
    order_store: Arc<CqrsOrderStore>,
}
// ANCHOR_END: container

// ANCHOR: factory
impl ShopContainerFactories for ShopContainer {
    fn build_order_store(inputs: ShopContainerOrderStoreInputs) -> Arc<CqrsOrderStore> {
        let ShopContainerOrderStoreInputs { read_db, write_db } = inputs;
        Arc::new(CqrsOrderStore::new(read_db, write_db))
    }
}
// ANCHOR_END: factory

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    println!("{}", container.get_order_store().describe_routing());
}
// ANCHOR_END: run
