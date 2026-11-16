use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use snapfit::container;

pub struct ShopConfig {
    pub store_name: &'static str,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self {
            store_name: "Northwind",
        }
    }
}

pub struct PaymentGateway {
    pub connected_to: &'static str,
}

impl PaymentGateway {
    pub async fn connect(config: Arc<ShopConfig>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            connected_to: config.store_name,
        })
    }
}

pub struct OrderService {
    gateway: Arc<PaymentGateway>,
}

impl OrderService {
    pub fn new(gateway: Arc<PaymentGateway>) -> Self {
        Self { gateway }
    }

    pub fn gateway_ref(&self) -> Arc<PaymentGateway> {
        Arc::clone(&self.gateway)
    }
}

#[container(mode = "async")]
pub struct AsyncShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], async, constructor = "connect")]
    gateway: Arc<PaymentGateway>,

    #[singleton(deps = [gateway])]
    orders: Arc<OrderService>,
}

static NEXT_CART_ID: AtomicU32 = AtomicU32::new(1);

pub struct Cart {
    pub id: u32,
    config: Arc<ShopConfig>,
}

impl Cart {
    pub fn new(config: Arc<ShopConfig>) -> Self {
        Self {
            id: NEXT_CART_ID.fetch_add(1, Ordering::Relaxed),
            config,
        }
    }

    pub fn config_ref(&self) -> Arc<ShopConfig> {
        Arc::clone(&self.config)
    }
}

#[container]
pub struct TransientShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[transient(deps = [config])]
    cart: Cart,
}

pub struct AuthService {
    pub realm: String,
}

#[container]
pub struct BuilderShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], builder = "build_auth")]
    auth: Arc<AuthService>,
}

impl BuilderShopContainer {
    fn build_auth(config: Arc<ShopConfig>) -> Arc<AuthService> {
        Arc::new(AuthService {
            realm: format!("{}-auth", config.store_name),
        })
    }
}

static NEXT_PRICE_LIST_ID: AtomicU32 = AtomicU32::new(1);

pub struct PriceList {
    pub id: u32,
}

impl PriceList {
    pub fn new() -> Self {
        Self {
            id: NEXT_PRICE_LIST_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

pub struct Quote {
    pub prices: PriceList,
}

impl Quote {
    pub fn new(prices: PriceList) -> Self {
        Self { prices }
    }
}

pub struct Catalog {
    pub prices: PriceList,
}

impl Catalog {
    pub fn new(prices: PriceList) -> Self {
        Self { prices }
    }
}

pub trait Notifier: Send + Sync {
    fn channel(&self) -> String;
}

struct EmailNotifier {
    sender: String,
}

impl Notifier for EmailNotifier {
    fn channel(&self) -> String {
        format!("email:{}", self.sender)
    }
}

/// A transient that feeds another transient and a singleton, plus a
/// trait-object singleton that only a builder can make.
#[container]
pub struct LayeredShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[transient]
    prices: PriceList,

    #[transient(deps = [prices])]
    quote: Quote,

    #[singleton(deps = [prices])]
    catalog: Arc<Catalog>,

    #[singleton(deps = [config], builder = "build_notifier")]
    notifier: Arc<dyn Notifier>,
}

impl LayeredShopContainer {
    fn build_notifier(config: Arc<ShopConfig>) -> Arc<dyn Notifier> {
        Arc::new(EmailNotifier {
            sender: config.store_name.to_lowercase(),
        })
    }
}
