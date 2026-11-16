#![deny(warnings)]
//! Part 2 Without Snapfit: TypeMap collision when registering two instances of the same type.

use snapfit::container;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

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

// ANCHOR: typemap_collision
#[derive(Default)]
pub struct TypeMapLocator {
    entries: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl TypeMapLocator {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self, instance: T) {
        self.entries.insert(TypeId::of::<T>(), Box::new(instance));
    }

    pub fn resolve<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.entries.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }
}
// ANCHOR_END: typemap_collision

// ANCHOR: snapfit_named_edges
#[container]
pub struct NamedDatabaseContainer {
    #[singleton(constructor = replica)]
    read_db: Arc<Database>,

    #[singleton(constructor = primary)]
    write_db: Arc<Database>,
}
// ANCHOR_END: snapfit_named_edges

// ANCHOR: run
fn main() {
    let mut locator = TypeMapLocator::new();
    locator.register(Arc::new(Database::replica()));
    locator.register(Arc::new(Database::primary()));

    let resolved = locator.resolve::<Arc<Database>>().unwrap();
    println!(
        "typemap kept {} entry: {}",
        locator.entries.len(),
        resolved.role()
    );

    let container = NamedDatabaseContainer::new();
    println!(
        "snapfit kept both: read={}, write={}",
        container.get_read_db().role(),
        container.get_write_db().role()
    );
}
// ANCHOR_END: run
