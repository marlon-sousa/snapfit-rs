#![deny(warnings)]
//! Chapter 12: a checkout whose test has to build the whole graph below it by hand.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: infrastructure
struct ShopConfig {
    currency: &'static str,
    kept_back: u32,
}

impl ShopConfig {
    fn new() -> Self {
        println!("reading the shop configuration");
        Self {
            currency: "EUR",
            kept_back: 1,
        }
    }
}

struct Database {
    url: &'static str,
}

impl Database {
    fn new() -> Self {
        Self::connect("postgres://shop")
    }

    fn connect(url: &'static str) -> Self {
        println!("opening a connection pool to {url}");
        Self { url }
    }

    fn stock_of(&self, item: &str) -> u32 {
        if item == "lamp" { 3 } else { 0 }
    }

    fn record_sale(&self, item: &str, cents: u64) -> String {
        format!("sale of {item} for {cents} cents recorded in {}", self.url)
    }
}
// ANCHOR_END: infrastructure

// ANCHOR: services
struct OrderService {
    config: Arc<ShopConfig>,
    db: Arc<Database>,
}

impl OrderService {
    fn new(config: Arc<ShopConfig>, db: Arc<Database>) -> Self {
        Self { config, db }
    }

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
    db: Arc<Database>,
}

impl StockService {
    fn new(config: Arc<ShopConfig>, db: Arc<Database>) -> Self {
        Self { config, db }
    }

    fn available(&self, item: &str) -> bool {
        self.db.stock_of(item) > self.config.kept_back
    }
}
// ANCHOR_END: services

// ANCHOR: controller
struct CheckoutController {
    orders: Arc<OrderService>,
    stock: Arc<StockService>,
}

impl CheckoutController {
    fn new(orders: Arc<OrderService>, stock: Arc<StockService>) -> Self {
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
// ANCHOR_END: controller

// ANCHOR: container
#[container]
struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton]
    db: Arc<Database>,

    #[singleton(deps = [config, db])]
    orders: Arc<OrderService>,

    #[singleton(deps = [config, db])]
    stock: Arc<StockService>,

    #[singleton(deps = [orders, stock])]
    checkout: Arc<CheckoutController>,
}
// ANCHOR_END: container

// ANCHOR: by_hand
fn checkout_refuses_an_item_out_of_stock() {
    let config = Arc::new(ShopConfig::new());
    let db = Arc::new(Database::connect("postgres://localhost/shop_test"));
    let orders = Arc::new(OrderService::new(config.clone(), db.clone()));
    let stock = Arc::new(StockService::new(config, db));
    let checkout = CheckoutController::new(orders, stock);

    assert_eq!(checkout.checkout("kettle", 2500), "kettle is out of stock");
}
// ANCHOR_END: by_hand

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();
    println!("{}", shop.get_checkout().checkout("lamp", 4500));

    println!("the test, by hand:");
    checkout_refuses_an_item_out_of_stock();
    println!("passed");
}
// ANCHOR_END: run
