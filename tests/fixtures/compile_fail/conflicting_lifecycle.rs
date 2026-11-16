use std::sync::Arc;
use snapfit::container;

pub struct Config;

#[container]
pub struct ConflictingLifecycleContainer {
    #[singleton]
    #[transient]
    config: Arc<Config>,
}
