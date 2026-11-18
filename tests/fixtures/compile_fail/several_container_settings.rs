use std::sync::Arc;
use snapfit::container;

pub struct Config;

#[container(mode = "threaded", leaf_build = factory)]
pub struct Shop {
    #[singleton]
    config: Arc<Config>,
}
