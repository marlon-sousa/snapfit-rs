use std::sync::Arc;
use snapfit::container;

pub trait Cache {}

#[container]
pub struct CacheContainer {
    #[singleton]
    cache: Arc<dyn Cache>,
}
