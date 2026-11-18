#![deny(warnings)]
//! Chapter 7: wiring prerequisites, named constructors, `Default` leaves, and Clone types without Arc.

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

#[derive(Clone, Default)]
pub struct DiscountCodes {
    pub codes: Vec<String>,
}

#[derive(Clone, Default)]
pub struct GiftWrapOptions {
    pub papers: Vec<String>,
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

    #[singleton(build = default)]
    discounts: DiscountCodes,

    #[singleton(deps = [tax], build = constructor(from_tax_table))]
    db: Arc<CatalogDb>,

    #[singleton(deps = [db], build = constructor(for_catalog))]
    shipping: Arc<ShippingClient>,
}
// ANCHOR_END: container

// ANCHOR: leaf_build
#[container(leaf_build = default)]
pub struct PromotionsContainer {
    #[singleton]
    discounts: DiscountCodes,

    #[singleton]
    gift_wrap: GiftWrapOptions,

    #[singleton(build = constructor(new))]
    tax: TaxRateTable,
}
// ANCHOR_END: leaf_build

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    println!("tax rate: {}%", container.get_tax().percentage);
    println!("database dsn: {}", container.get_db().dsn());
    println!("shipping endpoint: {}", container.get_shipping().endpoint());
    println!("discount codes: {}", container.get_discounts().codes.len());

    let promotions = PromotionsContainer::new();
    println!(
        "promotions: {} discount codes, {} gift papers, tax rate {}%",
        promotions.get_discounts().codes.len(),
        promotions.get_gift_wrap().papers.len(),
        promotions.get_tax().percentage
    );
}
// ANCHOR_END: run
