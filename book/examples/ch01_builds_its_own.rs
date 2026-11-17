#![deny(warnings)]
//! Chapter 1: services that build their own dependencies.

// ANCHOR: infrastructure
struct Database {
    url: &'static str,
}

impl Database {
    fn open(url: &'static str) -> Self {
        println!("opening a connection to {url}");
        Self { url }
    }

    fn url(&self) -> &'static str {
        self.url
    }
}

struct StripeGateway;

impl StripeGateway {
    fn connect(api_key: &str) -> Self {
        println!("connecting to Stripe with key {api_key}");
        Self
    }

    fn charge(&self, cents: u64) -> String {
        format!("Stripe charged {cents} cents")
    }
}
// ANCHOR_END: infrastructure

// ANCHOR: services
struct StockService {
    db: Database,
}

impl StockService {
    fn new() -> Self {
        Self {
            db: Database::open("postgres://shop"),
        }
    }

    fn reserve(&self, sku: &str) -> String {
        format!("reserved {sku} in {}", self.db.url())
    }
}

struct OrderService {
    stock: StockService,
    payments: StripeGateway,
}

impl OrderService {
    fn new() -> Self {
        Self {
            stock: StockService::new(),
            payments: StripeGateway::connect("sk_live_shop"),
        }
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
    db: Database,
}

impl ReportService {
    fn new() -> Self {
        Self {
            db: Database::open("postgres://shop"),
        }
    }

    fn daily_sales(&self) -> String {
        format!("sales report read from {}", self.db.url())
    }
}
// ANCHOR_END: services

// ANCHOR: run
fn main() {
    let orders = OrderService::new();
    let reports = ReportService::new();

    println!("{}", orders.place_order("RUST-BOOK-01", 4500));
    println!("{}", reports.daily_sales());
}
// ANCHOR_END: run
