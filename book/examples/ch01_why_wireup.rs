#![deny(warnings)]
//! Chapter 1 (with wireup): the same shop wired through a declarative container manifest.

use std::sync::Arc;
use wireup::container;

#[derive(Default)]
pub struct ShopConfig {
    pub store_name: &'static str,
    pub flat_discount_cents: u64,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self {
            store_name: "Northwind Books",
            flat_discount_cents: 500,
        }
    }
}

pub struct Database {
    config: Arc<ShopConfig>,
}

impl Database {
    pub fn new(config: Arc<ShopConfig>) -> Self {
        Self { config }
    }

    pub fn store_header(&self) -> &'static str {
        self.config.store_name
    }
}

pub struct StockService {
    db: Arc<Database>,
}

impl StockService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn reserve(&self, sku: &str) -> String {
        format!("[{}] reserved {sku}", self.db.store_header())
    }
}

pub struct OrderService {
    config: Arc<ShopConfig>,
    db: Arc<Database>,
    stock: Arc<StockService>,
}

impl OrderService {
    pub fn new(config: Arc<ShopConfig>, db: Arc<Database>, stock: Arc<StockService>) -> Self {
        Self { config, db, stock }
    }

    pub fn place_order(&self, sku: &str, subtotal_cents: u64) -> String {
        let total = subtotal_cents.saturating_sub(self.config.flat_discount_cents);
        let reservation = self.stock.reserve(sku);
        format!(
            "{reservation} | billed {total} cents via {}",
            self.db.store_header()
        )
    }
}

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config])]
    db: Arc<Database>,

    #[singleton(deps = [db])]
    stock: Arc<StockService>,

    #[transient(deps = [config, db, stock])]
    orders: OrderService,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    let orders = container.get_orders();
    println!("{}", orders.place_order("RUST-BOOK-01", 4500));
}
// ANCHOR_END: run
