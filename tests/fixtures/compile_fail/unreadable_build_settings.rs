use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Cache;

#[container(mode = "threaded", leaf_build = frobnicate, bogus)]
pub struct Shop {
    #[singleton]
    config: Arc<Config>,

    #[singleton(build = frobnicate, bogus)]
    cache: Arc<Cache>,
}
