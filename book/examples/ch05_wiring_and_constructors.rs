#![deny(warnings)]
//! Chapter 5: wiring prerequisites, custom constructors, custom builders, and Clone types without Arc.

use std::sync::Arc;
use wireup::container;

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
    pub fn custom(endpoint: String) -> Self {
        Self { endpoint }
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

    #[singleton(deps = [tax], constructor = "from_tax_table")]
    db: Arc<CatalogDb>,

    #[singleton(deps = [db], builder = "build_shipping")]
    shipping: Arc<ShippingClient>,
}

impl ShopContainer {
    fn build_shipping(db: Arc<CatalogDb>) -> Arc<ShippingClient> {
        Arc::new(ShippingClient::custom(format!(
            "https://ship.example.com?source={}",
            db.dsn()
        )))
    }
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
