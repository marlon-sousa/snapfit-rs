use std::sync::Arc;
use snapfit::container;

pub trait PaymentGateway {}
pub struct StripeGateway;
pub struct FakeGateway;

#[container]
pub struct ShopContainer {
    #[dispatch(StripeGateway, FakeGateway)]
    payments: Arc<dyn PaymentGateway>,
}
