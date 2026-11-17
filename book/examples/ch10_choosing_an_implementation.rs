#![deny(warnings)]
//! Chapter 10: the container chooses the implementation; the services only name a trait.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: gateways
pub trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

#[derive(Default)]
pub struct StripeGateway;

impl StripeGateway {
    pub fn new() -> Self {
        Self
    }
}

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
// ANCHOR_END: gateways

// ANCHOR: services
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

pub struct RefundService<P: PaymentGateway> {
    payments: Arc<P>,
}

impl<P: PaymentGateway> RefundService<P> {
    pub fn new(payments: Arc<P>) -> Self {
        Self { payments }
    }

    pub fn refund(&self, cents: u64) -> String {
        format!("refund: {}", self.payments.charge(cents))
    }
}
// ANCHOR_END: services

// ANCHOR: containers
#[container]
pub struct ShopContainer {
    #[singleton]
    payments: Arc<StripeGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,

    #[singleton(deps = [payments])]
    refunds: Arc<RefundService<StripeGateway>>,
}

#[container]
pub struct TestShopContainer {
    #[singleton]
    payments: Arc<FakeGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,

    #[singleton(deps = [payments])]
    refunds: Arc<RefundService<FakeGateway>>,
}
// ANCHOR_END: containers

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();
    println!("production {}", shop.get_orders().place_order(4500));
    println!("production {}", shop.get_refunds().refund(500));

    let test_shop = TestShopContainer::new();
    println!("test {}", test_shop.get_orders().place_order(4500));
    println!("test {}", test_shop.get_refunds().refund(500));
}
// ANCHOR_END: run
