#![deny(warnings)]
//! Chapter 15: adding a new domain capability touches only its struct and one container field.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: domain_step
#[derive(Default)]
pub struct ShopConfig {
    pub max_order_cents: u64,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self {
            max_order_cents: 50_000,
        }
    }
}

pub struct FraudGuard {
    config: Arc<ShopConfig>,
}

impl FraudGuard {
    pub fn new(config: Arc<ShopConfig>) -> Self {
        Self { config }
    }

    pub fn allows(&self, cents: u64) -> bool {
        cents <= self.config.max_order_cents
    }
}

pub struct CheckoutService {
    guard: Arc<FraudGuard>,
}

impl CheckoutService {
    pub fn new(guard: Arc<FraudGuard>) -> Self {
        Self { guard }
    }

    pub fn process(&self, order_id: u32, cents: u64) -> String {
        if self.guard.allows(cents) {
            format!("order #{order_id}: approved ({cents} cents)")
        } else {
            format!("order #{order_id}: held for review ({cents} cents)")
        }
    }
}
// ANCHOR_END: domain_step

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config])]
    guard: Arc<FraudGuard>,

    #[singleton(deps = [guard])]
    checkout: Arc<CheckoutService>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    let checkout = container.get_checkout();
    println!("{}", checkout.process(201, 12_000));
    println!("{}", checkout.process(202, 75_000));
}
// ANCHOR_END: run
