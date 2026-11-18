use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Store;
pub struct Cache;

#[container]
pub struct BrokenContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton]
    store: Arc<Store>,

    #[singleton(deps = [config, store, config, store, config])]
    cache: Arc<Cache>,
}
