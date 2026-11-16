use std::sync::Arc;
use snapfit::container;

pub struct Config;

#[container(mode = "sync")]
pub struct SyncWithAsyncDepContainer {
    #[singleton(async)]
    config: Arc<Config>,
}
