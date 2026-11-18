use snapfit::container;
use std::sync::Arc;

pub struct ShopConfig;

impl ShopConfig {
    pub fn new() -> Self {
        Self
    }
}

pub struct PaymentGateway;

impl PaymentGateway {
    pub fn connect(_config: Arc<ShopConfig>) -> Self {
        Self
    }
}

#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], build = constructor("connect"))]
    payments: Arc<PaymentGateway>,
}
