use std::sync::Arc;
use snapfit::container;

pub trait Store {}
pub struct Database;
pub struct ServiceA;
pub struct ServiceB;

#[container]
pub struct BrokenContainer {
    #[singleton(deps = [service_b])]
    service_a: Arc<ServiceA>,

    #[singleton(deps = [service_a])]
    service_b: Arc<ServiceB>,

    #[singleton(deps = [confog])]
    db: Arc<Database>,

    #[singleton]
    store: Arc<dyn Store>,
}
