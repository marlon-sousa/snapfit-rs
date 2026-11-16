#![deny(warnings)]
//! Chapter 4: leaf singletons are constructed once at boot and shared across getters.

use std::sync::Arc;
use wireup::container;

// ANCHOR: domain
#[derive(Default)]
pub struct ShopConfig {
    pub store_name: &'static str,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self {
            store_name: "Northwind Books",
        }
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    let first = container.get_config();
    let second = container.get_config();

    println!("store: {}", first.store_name);
    println!("same allocation: {}", Arc::ptr_eq(&first, &second));
}
// ANCHOR_END: run
