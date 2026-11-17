#![deny(warnings)]
//! Chapter 5: wiring prerequisites, named constructors, and Clone types without Arc.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: domain
#[derive(Clone, Default)]
pub struct TaxRateTable {
    pub percentage: u64,
}

impl TaxRateTable {
    pub fn new() -> Self {
        Self { percentage: 10 }
    }
}

pub struct CatalogDb {
    dsn: String,
}

impl CatalogDb {
    pub fn from_tax_table(tax: TaxRateTable) -> Self {
        Self {
            dsn: format!("postgres://shop?tax={}", tax.percentage),
        }
    }

    pub fn dsn(&self) -> &str {
        &self.dsn
    }
}

pub struct ShippingClient {
    endpoint: String,
}

impl ShippingClient {
    pub fn for_catalog(db: Arc<CatalogDb>) -> Self {
        Self {
            endpoint: format!("https://ship.example.com?source={}", db.dsn()),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton]
    tax: TaxRateTable,

    #[singleton(deps = [tax], constructor = from_tax_table)]
    db: Arc<CatalogDb>,

    #[singleton(deps = [db], constructor = for_catalog)]
    shipping: Arc<ShippingClient>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    println!("tax rate: {}%", container.get_tax().percentage);
    println!("database dsn: {}", container.get_db().dsn());
    println!("shipping endpoint: {}", container.get_shipping().endpoint());
}
// ANCHOR_END: run
