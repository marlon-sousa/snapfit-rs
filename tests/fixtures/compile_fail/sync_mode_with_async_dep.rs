use std::sync::Arc;
use wireup::container;

pub struct Config;

#[container(mode = "sync")]
pub struct SyncWithAsyncDepContainer {
    #[singleton(async)]
    config: Arc<Config>,
}
