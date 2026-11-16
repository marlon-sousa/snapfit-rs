#![deny(warnings)]
//! Chapter 3: reading the container reveals the full dependency graph regardless of field order.

use std::sync::Arc;
use wireup::container;

#[derive(Default)]
pub struct ShopConfig {
    pub region: &'static str,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self { region: "eu-west" }
    }
}

pub struct Database {
    config: Arc<ShopConfig>,
}

impl Database {
    pub fn new(config: Arc<ShopConfig>) -> Self {
        Self { config }
    }

    pub fn region(&self) -> &'static str {
        self.config.region
    }
}

pub struct PaymentClient {
    config: Arc<ShopConfig>,
}

impl PaymentClient {
    pub fn new(config: Arc<ShopConfig>) -> Self {
        Self { config }
    }

    pub fn charge(&self, cents: u64) -> String {
        format!("charged {cents} cents in {}", self.config.region)
    }
}

pub struct OrderRepository {
    db: Arc<Database>,
}

impl OrderRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn save(&self, order_id: u32) -> String {
        format!("saved order #{order_id} in {}", self.db.region())
    }
}

pub struct CheckoutHandler {
    repo: Arc<OrderRepository>,
    payments: Arc<PaymentClient>,
}

impl CheckoutHandler {
    pub fn new(repo: Arc<OrderRepository>, payments: Arc<PaymentClient>) -> Self {
        Self { repo, payments }
    }

    pub fn handle(&self, order_id: u32, cents: u64) -> String {
        format!(
            "{} | {}",
            self.payments.charge(cents),
            self.repo.save(order_id)
        )
    }
}

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[transient(deps = [repo, payments])]
    checkout: CheckoutHandler,

    #[singleton(deps = [db])]
    repo: Arc<OrderRepository>,

    #[singleton(deps = [config])]
    payments: Arc<PaymentClient>,

    #[singleton(deps = [config])]
    db: Arc<Database>,

    #[singleton]
    config: Arc<ShopConfig>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    let checkout = container.get_checkout();
    println!("{}", checkout.handle(1042, 2999));
}
// ANCHOR_END: run
