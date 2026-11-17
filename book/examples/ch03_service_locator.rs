#![deny(warnings)]
//! Chapter 3: a service locator keyed by type, and the mistakes it finds only at run time.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

struct Database {
    role: &'static str,
}

impl Database {
    fn primary() -> Self {
        Self { role: "primary" }
    }

    fn replica() -> Self {
        Self { role: "replica" }
    }
}

struct StripeGateway;

// ANCHOR: locator
#[derive(Default)]
struct ServiceLocator {
    entries: HashMap<TypeId, Box<dyn Any>>,
}

impl ServiceLocator {
    fn register<T: 'static>(&mut self, instance: T) {
        self.entries.insert(TypeId::of::<T>(), Box::new(instance));
    }

    fn resolve<T: 'static>(&self) -> Option<&T> {
        self.entries.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }
}
// ANCHOR_END: locator

// ANCHOR: run
fn main() {
    let mut locator = ServiceLocator::default();
    locator.register(Arc::new(Database::replica()));
    locator.register(Arc::new(Database::primary()));
    let db = locator
        .resolve::<Arc<Database>>()
        .map_or("none", |db| db.role);
    println!(
        "registered 2 databases; entries kept: {}; resolved: {db}",
        locator.entries.len()
    );

    locator.register(StripeGateway);
    let payments = locator.resolve::<Arc<StripeGateway>>();
    println!(
        "registered StripeGateway, asked for Arc<StripeGateway>; found: {}",
        payments.is_some()
    );
}
// ANCHOR_END: run
