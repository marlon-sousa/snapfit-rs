use snapfit::container;
use std::sync::Arc;

pub trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

pub struct OrderService;

impl OrderService {
    pub fn new(_payments: Arc<dyn PaymentGateway>) -> Self {
        Self
    }
}

#[container]
pub struct ShopContainer {
    #[singleton]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}
