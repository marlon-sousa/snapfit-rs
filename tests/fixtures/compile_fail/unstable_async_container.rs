use std::sync::Arc;
use snapfit::container;

pub struct Config;

#[container(mode = "async")]
pub struct AsyncModeContainer {
    #[singleton]
    config: Arc<Config>,
}
