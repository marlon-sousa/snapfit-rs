use std::sync::Arc;
use wireup::container;

pub struct PaymentGateway;

#[container(mode = "sync")]
pub struct SyncContainerWithAsyncDep {
    #[singleton(async, constructor = "connect")]
    gateway: Arc<PaymentGateway>,
}
