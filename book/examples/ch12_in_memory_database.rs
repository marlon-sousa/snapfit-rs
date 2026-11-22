#![deny(warnings)]
//! Chapter 12: an in-memory database behind the trait, named by a container for tests.

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

trait Database: Send + Sync {
    fn stock_of(&self, item: &str) -> u32;
    fn record_sale(&self, item: &str, cents: u64) -> String;
}

// ANCHOR: in_memory
struct InMemoryDatabase {
    stock: Vec<(&'static str, u32)>,
}

impl InMemoryDatabase {
    fn new() -> Self {
        Self {
            stock: vec![("lamp", 1), ("kettle", 5)],
        }
    }
}

impl Database for InMemoryDatabase {
    fn stock_of(&self, item: &str) -> u32 {
        self.stock
            .iter()
            .find(|(name, _)| *name == item)
            .map_or(0, |(_, count)| *count)
    }

    fn record_sale(&self, item: &str, cents: u64) -> String {
        format!("sale of {item} for {cents} cents kept in memory")
    }
}
// ANCHOR_END: in_memory

trait Orders {
    fn place(&self, item: &str, cents: u64) -> String;
}

trait Stock {
    fn available(&self, item: &str) -> bool;
}

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

// ANCHOR: test_container
#[container]
struct TestShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(implementation = InMemoryDatabase)]
    db: Arc<dyn Database>,

    #[singleton(implementation = OrderService, deps = [config, db])]
    orders: Arc<dyn Orders>,

    #[singleton(implementation = StockService, deps = [config, db])]
    stock: Arc<dyn Stock>,

    #[singleton(deps = [orders, stock])]
    checkout: Arc<CheckoutController>,
}
// ANCHOR_END: test_container

// ANCHOR: test
fn checkout_keeps_the_last_item_back() {
    let shop = TestShopContainer::new();
    let checkout = shop.get_checkout();

    assert_eq!(checkout.checkout("lamp", 4500), "lamp is out of stock");
    assert_eq!(
        checkout.checkout("kettle", 2500),
        "sale of kettle for 2500 cents kept in memory, in EUR"
    );
}
// ANCHOR_END: test

// ANCHOR: run
fn main() {
    println!("the test, through the real services:");
    checkout_keeps_the_last_item_back();
    println!("checkout_keeps_the_last_item_back passed");
}
// ANCHOR_END: run
