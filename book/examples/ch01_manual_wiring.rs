#![deny(warnings)]
//! Chapter 1 (usual way): manual constructor threading across layers without wireup.

use std::sync::Arc;

// ANCHOR: domain
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
// ANCHOR_END: domain

// ANCHOR: manual_assembly
pub struct ManualAppState {
    config: Arc<ShopConfig>,
    db: Arc<Database>,
    stock: Arc<StockService>,
}

impl ManualAppState {
    pub fn build() -> Self {
        let config = Arc::new(ShopConfig::new());
        let db = Arc::new(Database::new(Arc::clone(&config)));
        let stock = Arc::new(StockService::new(Arc::clone(&db)));
        Self { config, db, stock }
    }

    pub fn build_order_service(&self) -> OrderService {
        OrderService::new(
            Arc::clone(&self.config),
            Arc::clone(&self.db),
            Arc::clone(&self.stock),
        )
    }
}
// ANCHOR_END: manual_assembly

// ANCHOR: run
fn main() {
    let state = ManualAppState::build();
    let orders = state.build_order_service();
    println!("{}", orders.place_order("RUST-BOOK-01", 4500));
}
// ANCHOR_END: run
