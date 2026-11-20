#![deny(warnings)]
//! Chapter 11: services generic over the gateway, and an alias that keeps a swap to one line.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: trait
pub trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
    fn provider(&self) -> &'static str;
}
// ANCHOR_END: trait

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

    fn provider(&self) -> &'static str {
        "Stripe"
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

    fn provider(&self) -> &'static str {
        "the fake gateway"
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

// ANCHOR: receipts
pub struct ReceiptPrinter {
    provider: &'static str,
}

impl ReceiptPrinter {
    pub fn new<P: PaymentGateway + ?Sized>(payments: Arc<P>) -> Self {
        Self {
            provider: payments.provider(),
        }
    }

    pub fn print(&self, cents: u64) -> String {
        format!("receipt: {cents} cents, paid through {}", self.provider)
    }
}
// ANCHOR_END: receipts

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

// ANCHOR: container
pub type ShopGateway = StripeGateway;

#[container]
pub struct ShopContainer {
    #[singleton]
    payments: Arc<ShopGateway>,

    #[singleton(deps = [payments])]
    refunds: Arc<RefundService<ShopGateway>>,

    #[singleton(deps = [payments])]
    receipts: Arc<ReceiptPrinter>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}
// ANCHOR_END: container

// ANCHOR: test_container
pub type TestShopGateway = FakeGateway;

#[container]
pub struct TestShopContainer {
    #[singleton]
    payments: Arc<TestShopGateway>,

    #[singleton(deps = [payments])]
    refunds: Arc<RefundService<TestShopGateway>>,

    #[singleton(deps = [payments])]
    receipts: Arc<ReceiptPrinter>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}
// ANCHOR_END: test_container

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();
    println!("production {}", shop.get_refunds().refund(500));
    println!("production {}", shop.get_receipts().print(4500));
    println!("production {}", shop.get_orders().place_order(9000));

    let test_shop = TestShopContainer::new();
    println!("test {}", test_shop.get_refunds().refund(500));
    println!("test {}", test_shop.get_receipts().print(4500));
    println!("test {}", test_shop.get_orders().place_order(9000));
}
// ANCHOR_END: run
