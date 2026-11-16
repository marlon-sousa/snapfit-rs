#![deny(warnings)]
//! Chapter 8: eager asynchronous container boot with synchronous, lock-free getters.

use std::error::Error;
use std::sync::Arc;
use wireup::container;

// ANCHOR: domain
#[derive(Default)]
pub struct ShopConfig {
    pub payment_url: &'static str,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self {
            payment_url: "https://pay.northwind.example",
        }
    }
}

pub struct PaymentGateway {
    endpoint: &'static str,
}

impl PaymentGateway {
    pub async fn connect(config: Arc<ShopConfig>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            endpoint: config.payment_url,
        })
    }

    pub fn authorize(&self, cents: u64) -> String {
        format!("authorized {cents} cents via {}", self.endpoint)
    }
}

pub struct OrderService {
    gateway: Arc<PaymentGateway>,
}

impl OrderService {
    pub fn new(gateway: Arc<PaymentGateway>) -> Self {
        Self { gateway }
    }

    pub fn checkout(&self, cents: u64) -> String {
        self.gateway.authorize(cents)
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container(mode = "async")]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], async, constructor = "connect")]
    gateway: Arc<PaymentGateway>,

    #[singleton(deps = [gateway])]
    orders: Arc<OrderService>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    futures::executor::block_on(async {
        let container = ShopContainer::init().await?;
        let orders = container.get_orders();
        println!("{}", orders.checkout(6400));
        Ok(())
    })
}
// ANCHOR_END: run
