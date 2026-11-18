use std::sync::Arc;
use snapfit::container;

pub trait Clock {}

#[container]
pub struct Shop {
    #[singleton(build = factory)]
    clock: Arc<dyn Clock>,
}
