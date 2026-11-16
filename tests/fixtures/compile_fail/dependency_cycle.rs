use std::sync::Arc;
use snapfit::container;

pub struct ServiceA;
pub struct ServiceB;

#[container]
pub struct CyclicContainer {
    #[singleton(deps = [service_b])]
    service_a: Arc<ServiceA>,

    #[singleton(deps = [service_a])]
    service_b: Arc<ServiceB>,
}
