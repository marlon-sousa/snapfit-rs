use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Cache;

#[derive(bon::Builder)]
pub struct Pool {
    reader: Arc<Config>,
    writer: Arc<Config>,
}

#[container]
pub struct BrokenContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [nosuch, nosuch])]
    cache: Arc<Cache>,

    #[singleton(build = builder, deps = [reader = nosuch, writer = nosuch])]
    pool: Arc<Pool>,
}
