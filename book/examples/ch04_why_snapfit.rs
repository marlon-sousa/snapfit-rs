#![deny(warnings)]
//! Chapter 4: the shop of chapter 3, built by a snapfit container instead of by hand.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: domain
#[derive(Default)]
pub struct ShopConfig {
    pub database_url: &'static str,
    pub stripe_key: &'static str,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self {
            database_url: "postgres://shop",
            stripe_key: "sk_live_shop",
        }
    }
}

pub struct Database {
    url: &'static str,
}

impl Database {
    pub fn open(config: Arc<ShopConfig>) -> Self {
        println!("opening a connection to {}", config.database_url);
        Self {
            url: config.database_url,
        }
    }
}

pub trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

pub struct StripeGateway;

impl StripeGateway {
    pub fn connect(config: Arc<ShopConfig>) -> Self {
        println!("connecting to Stripe with key {}", config.stripe_key);
        Self
    }
}

impl PaymentGateway for StripeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("Stripe charged {cents} cents")
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
        format!("reserved {sku} in {}", self.db.url)
    }
}

pub struct OrderService {
    stock: Arc<StockService>,
    payments: Arc<dyn PaymentGateway>,
}

impl OrderService {
    pub fn new(stock: Arc<StockService>, payments: Arc<dyn PaymentGateway>) -> Self {
        Self { stock, payments }
    }

    pub fn place_order(&self, sku: &str, cents: u64) -> String {
        format!(
            "{}; {}",
            self.stock.reserve(sku),
            self.payments.charge(cents)
        )
    }
}

pub struct ReportService {
    db: Arc<Database>,
}

impl ReportService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn daily_sales(&self) -> String {
        format!("sales report read from {}", self.db.url)
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], constructor = open)]
    db: Arc<Database>,

    #[singleton(deps = [config], constructor = connect)]
    payments: Arc<StripeGateway>,

    #[singleton(deps = [db])]
    stock: Arc<StockService>,

    #[singleton(deps = [stock, payments])]
    orders: Arc<OrderService>,

    #[singleton(deps = [db])]
    reports: Arc<ReportService>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();

    println!("{}", shop.get_orders().place_order("RUST-BOOK-01", 4500));
    println!("{}", shop.get_reports().daily_sales());
    println!(
        "the same order service every time: {}",
        Arc::ptr_eq(&shop.get_orders(), &shop.get_orders())
    );
}
// ANCHOR_END: run
