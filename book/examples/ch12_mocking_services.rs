#![deny(warnings)]
//! Chapter 12: a checkout that asks for trait objects, tested with mockall's stand-ins.

use mockall::automock;
use snapfit::container;
use std::sync::Arc;

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
        println!("opening a connection pool to postgres://shop");
        Self {
            url: "postgres://shop",
        }
    }

    fn stock_of(&self, item: &str) -> u32 {
        if item == "lamp" { 3 } else { 0 }
    }

    fn record_sale(&self, item: &str, cents: u64) -> String {
        format!("sale of {item} for {cents} cents recorded in {}", self.url)
    }
}

// ANCHOR: traits
#[automock]
trait Orders: Send + Sync {
    fn place(&self, item: &str, cents: u64) -> String;
}

#[automock]
trait Stock: Send + Sync {
    fn available(&self, item: &str) -> bool;
}
// ANCHOR_END: traits

// ANCHOR: services
struct OrderService {
    config: Arc<ShopConfig>,
    db: Arc<Database>,
}

impl OrderService {
    fn new(config: Arc<ShopConfig>, db: Arc<Database>) -> Self {
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
    db: Arc<Database>,
}

impl StockService {
    fn new(config: Arc<ShopConfig>, db: Arc<Database>) -> Self {
        Self { config, db }
    }
}

impl Stock for StockService {
    fn available(&self, item: &str) -> bool {
        self.db.stock_of(item) > self.config.kept_back
    }
}
// ANCHOR_END: services

// ANCHOR: controller
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
// ANCHOR_END: controller

// ANCHOR: container
#[container]
struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton]
    db: Arc<Database>,

    #[singleton(implementation = OrderService, deps = [config, db])]
    orders: Arc<dyn Orders>,

    #[singleton(implementation = StockService, deps = [config, db])]
    stock: Arc<dyn Stock>,

    #[singleton(deps = [orders, stock])]
    checkout: Arc<CheckoutController>,
}
// ANCHOR_END: container

// ANCHOR: test_out_of_stock
fn checkout_refuses_an_item_out_of_stock() {
    let mut stock = MockStock::new();
    stock
        .expect_available()
        .withf(|item| item == "kettle")
        .times(1)
        .return_const(false);

    let mut orders = MockOrders::new();
    orders.expect_place().never();

    let checkout = CheckoutController::new(Arc::new(orders), Arc::new(stock));

    assert_eq!(checkout.checkout("kettle", 2500), "kettle is out of stock");
}
// ANCHOR_END: test_out_of_stock

// ANCHOR: test_in_stock
fn checkout_places_an_item_in_stock() {
    let mut stock = MockStock::new();
    stock.expect_available().return_const(true);

    let mut orders = MockOrders::new();
    orders
        .expect_place()
        .withf(|item, cents| item == "lamp" && *cents == 4500)
        .times(1)
        .returning(|item, cents| format!("placed {item} for {cents} cents"));

    let checkout = CheckoutController::new(Arc::new(orders), Arc::new(stock));

    assert_eq!(
        checkout.checkout("lamp", 4500),
        "placed lamp for 4500 cents"
    );
}
// ANCHOR_END: test_in_stock

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();
    println!("{}", shop.get_checkout().checkout("lamp", 4500));

    println!("the tests, with stand-ins:");
    checkout_refuses_an_item_out_of_stock();
    println!("checkout_refuses_an_item_out_of_stock passed");
    checkout_places_an_item_in_stock();
    println!("checkout_places_an_item_in_stock passed");
}
// ANCHOR_END: run
