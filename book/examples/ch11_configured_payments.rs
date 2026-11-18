#![deny(warnings)]
//! Chapter 11: a field typed as a trait object, built by a factory that reads the configuration.

use snapfit::container;
use std::sync::Arc;

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
    let shop = ConfiguredShopContainer::new();
    println!("configured {}", shop.get_orders().place_order(4500));
    println!("configured gateway: {}", shop.get_payments().charge(100));
    factory_alone();
}
// ANCHOR_END: run

// ANCHOR: factory_alone
fn factory_alone() {
    let inputs = ConfiguredShopContainerPaymentsInputs {
        config: ShopConfig {
            payments: PaymentProvider::Stripe,
        },
    };
    let gateway = ConfiguredShopContainer::build_payments(inputs);
    println!("factory alone: {}", gateway.charge(100));
}
// ANCHOR_END: factory_alone
