#![deny(warnings)]
//! Chapter 9: multiple instances of the exact same type wired by field name.

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

    #[singleton(deps = [read_db, write_db])]
    order_store: Arc<CqrsOrderStore>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    let read_db = container.get_read_db();
    let write_db = container.get_write_db();

    println!("{}", container.get_order_store().describe_routing());
    println!(
        "distinct allocations: {}",
        !Arc::ptr_eq(&read_db, &write_db)
    );
}
// ANCHOR_END: run
