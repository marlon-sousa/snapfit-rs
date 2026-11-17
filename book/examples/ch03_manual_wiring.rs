#![deny(warnings)]
//! Chapter 3: services receive their dependencies, and `AppState` builds and connects them by hand.

use std::sync::Arc;

// ANCHOR: domain
struct Database {
    url: &'static str,
}

impl Database {
    fn open(url: &'static str) -> Self {
        println!("opening a connection to {url}");
        Self { url }
    }
}

trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

struct StripeGateway;

impl StripeGateway {
    fn connect(api_key: &str) -> Self {
        println!("connecting to Stripe with key {api_key}");
        Self
    }
}

impl PaymentGateway for StripeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("Stripe charged {cents} cents")
    }
}

struct StockService {
    db: Arc<Database>,
}

impl StockService {
    fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn reserve(&self, sku: &str) -> String {
        format!("reserved {sku} in {}", self.db.url)
    }
}

struct OrderService {
    stock: Arc<StockService>,
    payments: Arc<dyn PaymentGateway>,
}

impl OrderService {
    fn new(stock: Arc<StockService>, payments: Arc<dyn PaymentGateway>) -> Self {
        Self { stock, payments }
    }

    fn place_order(&self, sku: &str, cents: u64) -> String {
        format!(
            "{}; {}",
            self.stock.reserve(sku),
            self.payments.charge(cents)
        )
    }
}

struct ReportService {
    db: Arc<Database>,
}

impl ReportService {
    fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn daily_sales(&self) -> String {
        format!("sales report read from {}", self.db.url)
    }
}
// ANCHOR_END: domain

// ANCHOR: manual_assembly
struct AppState {
    stock: Arc<StockService>,
    payments: Arc<dyn PaymentGateway>,
    reports: Arc<ReportService>,
}

impl AppState {
    fn build() -> Self {
        let db = Arc::new(Database::open("postgres://shop"));
        let stock = Arc::new(StockService::new(Arc::clone(&db)));
        let payments: Arc<dyn PaymentGateway> = Arc::new(StripeGateway::connect("sk_live_shop"));
        let reports = Arc::new(ReportService::new(Arc::clone(&db)));
        Self {
            stock,
            payments,
            reports,
        }
    }

    fn order_service(&self) -> OrderService {
        OrderService::new(Arc::clone(&self.stock), Arc::clone(&self.payments))
    }
}
// ANCHOR_END: manual_assembly

// ANCHOR: run
fn main() {
    let state = AppState::build();

    println!(
        "{}",
        state.order_service().place_order("RUST-BOOK-01", 4500)
    );
    println!("{}", state.reports.daily_sales());
}
// ANCHOR_END: run
