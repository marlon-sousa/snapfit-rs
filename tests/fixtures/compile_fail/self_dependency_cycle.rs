use std::sync::Arc;
use snapfit::container;

pub struct ServiceA;

#[container]
pub struct SelfCycleContainer {
    #[singleton(deps = [service_a])]
    service_a: Arc<ServiceA>,
}
