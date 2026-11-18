#![deny(warnings)]
//! Chapter 11: the container chooses the implementation; the services only name a trait.

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

// ANCHOR: config
#[derive(Clone, Copy, Default)]
pub enum PaymentProvider {
    Stripe,
    #[default]
    Fake,
}

#[derive(Clone, Default)]
pub struct ShopConfig {
    pub payments: PaymentProvider,
}
// ANCHOR_END: config

// ANCHOR: configured
#[container]
pub struct ConfiguredShopContainer {
    #[singleton(build = default)]
    config: ShopConfig,

    #[singleton(deps = [config], build = factory)]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}

impl ConfiguredShopContainerFactories for ConfiguredShopContainer {
    fn build_payments(inputs: ConfiguredShopContainerPaymentsInputs) -> Arc<dyn PaymentGateway> {
        let ConfiguredShopContainerPaymentsInputs { config } = inputs;
        match config.payments {
            PaymentProvider::Stripe => Arc::new(StripeGateway::new()),
            PaymentProvider::Fake => Arc::new(FakeGateway::new()),
        }
    }
}
// ANCHOR_END: configured

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();
    println!("production {}", shop.get_orders().place_order(4500));
    println!("production {}", shop.get_refunds().refund(500));

    let test_shop = TestShopContainer::new();
    println!("test {}", test_shop.get_orders().place_order(4500));
    println!("test {}", test_shop.get_refunds().refund(500));

    let configured = ConfiguredShopContainer::new();
    println!("configured {}", configured.get_orders().place_order(4500));
    println!("configured gateway: {}", configured.get_payments().charge(100));

    let stripe_config = ShopConfig {
        payments: PaymentProvider::Stripe,
    };
    let inputs = ConfiguredShopContainerPaymentsInputs {
        config: stripe_config,
    };
    let gateway = ConfiguredShopContainer::build_payments(inputs);
    println!("factory alone: {}", gateway.charge(100));
}
// ANCHOR_END: run
