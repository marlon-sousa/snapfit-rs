#![deny(warnings)]
//! Quick start: pure domain types wired by a single declarative container.

use std::sync::Arc;
use wireup::container;

// ANCHOR: domain
#[derive(Default)]
pub struct ShopConfig {
    pub currency: &'static str,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self { currency: "USD" }
    }
}

pub struct CatalogDb {
    config: Arc<ShopConfig>,
}

impl CatalogDb {
    pub fn new(config: Arc<ShopConfig>) -> Self {
        Self { config }
    }

    pub fn format_price(&self, cents: u64) -> String {
        format!(
            "{}.{:02} {}",
            cents / 100,
            cents % 100,
            self.config.currency
        )
    }
}

pub struct OrderCheckout {
    db: Arc<CatalogDb>,
}

impl OrderCheckout {
    pub fn new(db: Arc<CatalogDb>) -> Self {
        Self { db }
    }

    pub fn receipt(&self, item: &str, cents: u64) -> String {
        format!("charged {} for {item}", self.db.format_price(cents))
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config])]
    db: Arc<CatalogDb>,

    #[transient(deps = [db])]
    checkout: OrderCheckout,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    let checkout = container.get_checkout();
    println!("{}", checkout.receipt("mechanical keyboard", 14500));
}
// ANCHOR_END: run
