use std::sync::Arc;
use wireup::container;

pub struct Config;

#[container]
pub struct ConflictingLifecycleContainer {
    #[singleton]
    #[transient]
    config: Arc<Config>,
}
