#![deny(warnings)]
//! Chapter 11: a field typed as the trait, built from the implementation the container names.

use snapfit::container;
use std::sync::Arc;

pub trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

// ANCHOR: stripe
#[derive(Default)]
pub struct StripeGateway;

impl StripeGateway {
    pub fn new() -> Self {
        Self
    }

    pub fn dashboard_url(&self) -> &'static str {
        "https://dashboard.stripe.com"
    }
}
// ANCHOR_END: stripe

impl PaymentGateway for StripeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("Stripe charged {cents} cents")
    }
}

#[derive(Default)]
pub struct FakeGateway;

impl FakeGateway {
    pub fn new() -> Self {
        Self
    }
}

impl PaymentGateway for FakeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("fake gateway pretended to charge {cents} cents")
    }
}

pub struct OrderService {
    payments: Arc<dyn PaymentGateway>,
}

impl OrderService {
    pub fn new(payments: Arc<dyn PaymentGateway>) -> Self {
        Self { payments }
    }

    pub fn place_order(&self, cents: u64) -> String {
        format!("order: {}", self.payments.charge(cents))
    }
}

// ANCHOR: containers
#[container]
pub struct ShopContainer {
    #[singleton(implementation = StripeGateway)]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}

#[container]
pub struct TestShopContainer {
    #[singleton(implementation = FakeGateway)]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}
// ANCHOR_END: containers

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();
    println!("production {}", shop.get_orders().place_order(4500));

    let test_shop = TestShopContainer::new();
    println!("test {}", test_shop.get_orders().place_order(4500));
}
// ANCHOR_END: run
