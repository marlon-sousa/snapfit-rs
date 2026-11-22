#![deny(warnings)]
//! Chapter 12: a database behind a trait, swapped for another in one word.

use snapfit::container;
use std::sync::Arc;

struct ShopConfig {
    currency: &'static str,
    kept_back: u32,
}

impl ShopConfig {
    fn new() -> Self {
        Self {
            currency: "EUR",
            kept_back: 1,
        }
    }
}

// ANCHOR: trait
trait Database: Send + Sync {
    fn stock_of(&self, item: &str) -> u32;
    fn record_sale(&self, item: &str, cents: u64) -> String;
}
// ANCHOR_END: trait

// ANCHOR: postgres
struct PostgresDatabase;

impl PostgresDatabase {
    fn new() -> Self {
        println!("opening a connection pool to postgres://shop");
        Self
    }
}

impl Database for PostgresDatabase {
    fn stock_of(&self, item: &str) -> u32 {
        if item == "lamp" { 3 } else { 0 }
    }

    fn record_sale(&self, item: &str, cents: u64) -> String {
        format!("sale of {item} for {cents} cents recorded in PostgreSQL")
    }
}
// ANCHOR_END: postgres

// ANCHOR: sqlite
struct SqliteDatabase;

impl SqliteDatabase {
    fn new() -> Self {
        println!("opening shop.db with SQLite");
        Self
    }
}

impl Database for SqliteDatabase {
    fn stock_of(&self, item: &str) -> u32 {
        if item == "lamp" { 3 } else { 0 }
    }

    fn record_sale(&self, item: &str, cents: u64) -> String {
        format!("sale of {item} for {cents} cents recorded in SQLite")
    }
}
// ANCHOR_END: sqlite

trait Orders {
    fn place(&self, item: &str, cents: u64) -> String;
}

trait Stock {
    fn available(&self, item: &str) -> bool;
}

// ANCHOR: services
struct OrderService {
    config: Arc<ShopConfig>,
    db: Arc<dyn Database>,
}

impl OrderService {
    fn new(config: Arc<ShopConfig>, db: Arc<dyn Database>) -> Self {
        Self { config, db }
    }
}

impl Orders for OrderService {
    fn place(&self, item: &str, cents: u64) -> String {
        format!(
            "{}, in {}",
            self.db.record_sale(item, cents),
            self.config.currency
        )
    }
}

struct StockService {
    config: Arc<ShopConfig>,
    db: Arc<dyn Database>,
}

impl StockService {
    fn new(config: Arc<ShopConfig>, db: Arc<dyn Database>) -> Self {
        Self { config, db }
    }
}

impl Stock for StockService {
    fn available(&self, item: &str) -> bool {
        self.db.stock_of(item) > self.config.kept_back
    }
}
// ANCHOR_END: services

struct CheckoutController {
    orders: Arc<dyn Orders>,
    stock: Arc<dyn Stock>,
}

impl CheckoutController {
    fn new(orders: Arc<dyn Orders>, stock: Arc<dyn Stock>) -> Self {
        Self { orders, stock }
    }

    fn checkout(&self, item: &str, cents: u64) -> String {
        if self.stock.available(item) {
            self.orders.place(item, cents)
        } else {
            format!("{item} is out of stock")
        }
    }
}

// ANCHOR: before
#[container]
struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(implementation = PostgresDatabase)]
    db: Arc<dyn Database>,

    #[singleton(implementation = OrderService, deps = [config, db])]
    orders: Arc<dyn Orders>,

    #[singleton(implementation = StockService, deps = [config, db])]
    stock: Arc<dyn Stock>,

    #[singleton(deps = [orders, stock])]
    checkout: Arc<CheckoutController>,
}
// ANCHOR_END: before

// ANCHOR: after
#[container]
struct MovedShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(implementation = SqliteDatabase)]
    db: Arc<dyn Database>,

    #[singleton(implementation = OrderService, deps = [config, db])]
    orders: Arc<dyn Orders>,

    #[singleton(implementation = StockService, deps = [config, db])]
    stock: Arc<dyn Stock>,

    #[singleton(deps = [orders, stock])]
    checkout: Arc<CheckoutController>,
}
// ANCHOR_END: after

// ANCHOR: run
fn main() {
    let before = ShopContainer::new();
    println!("{}", before.get_checkout().checkout("lamp", 4500));

    let after = MovedShopContainer::new();
    println!("{}", after.get_checkout().checkout("lamp", 4500));
}
// ANCHOR_END: run
