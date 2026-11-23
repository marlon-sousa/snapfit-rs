#![deny(warnings)]
//! Chapter 13: fields whose constructor or factory can fail, and a container built by `try_new()`.

use snapfit::container;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// ANCHOR: network
/// Stands in for the network: whether the database answers when the shop starts.
static DATABASE_ANSWERS: AtomicBool = AtomicBool::new(true);
// ANCHOR_END: network

// ANCHOR: config
pub struct ShopConfig {
    pub database_address: String,
    pub payment_provider: String,
}

impl ShopConfig {
    pub fn load() -> Self {
        Self {
            database_address: "orders.db.internal:5432".to_string(),
            payment_provider: "stripe".to_string(),
        }
    }
}
// ANCHOR_END: config

// ANCHOR: database
#[derive(Debug)]
pub struct ConnectError {
    pub address: String,
}

impl fmt::Display for ConnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "nothing answered at {}", self.address)
    }
}

impl Error for ConnectError {}

pub struct Database {
    address: String,
}

impl Database {
    pub fn connect(config: Arc<ShopConfig>) -> Result<Self, ConnectError> {
        let address = config.database_address.clone();
        if DATABASE_ANSWERS.load(Ordering::SeqCst) {
            Ok(Self { address })
        } else {
            Err(ConnectError { address })
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}
// ANCHOR_END: database

pub trait PaymentGateway {
    fn name(&self) -> &'static str;
}

pub struct StripeGateway;

impl PaymentGateway for StripeGateway {
    fn name(&self) -> &'static str {
        "Stripe"
    }
}

pub struct OrderService {
    db: Arc<Database>,
    payments: Arc<dyn PaymentGateway>,
}

impl OrderService {
    pub fn new(db: Arc<Database>, payments: Arc<dyn PaymentGateway>) -> Self {
        Self { db, payments }
    }

    pub fn describe(&self) -> String {
        format!(
            "orders are stored at {} and paid through {}",
            self.db.address(),
            self.payments.name()
        )
    }
}

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton(build = constructor(load))]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], fallible = true, build = constructor(connect))]
    db: Arc<Database>,

    #[singleton(deps = [config], fallible = true, build = factory)]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(deps = [db, payments])]
    orders: Arc<OrderService>,
}
// ANCHOR_END: container

// ANCHOR: factory
impl ShopContainerFactories for ShopContainer {
    fn build_payments(
        inputs: ShopContainerPaymentsInputs,
    ) -> Result<Arc<dyn PaymentGateway>, Box<dyn Error + Send + Sync>> {
        let ShopContainerPaymentsInputs { config } = inputs;
        match config.payment_provider.as_str() {
            "stripe" => Ok(Arc::new(StripeGateway)),
            other => Err(format!("no payment provider is called {other}").into()),
        }
    }
}
// ANCHOR_END: factory

// ANCHOR: run
fn main() {
    open_the_shop();
    DATABASE_ANSWERS.store(false, Ordering::SeqCst);
    open_the_shop();
}

fn open_the_shop() {
    match ShopContainer::try_new() {
        Ok(shop) => println!("The shop is open: {}.", shop.get_orders().describe()),
        Err(error) => {
            println!("The shop could not open: {error}.");
            println!("The field that failed: {}.", error.field());
            if let Some(cause) = error.source() {
                println!("Its own error: {cause}.");
            }
        }
    }
}
// ANCHOR_END: run
