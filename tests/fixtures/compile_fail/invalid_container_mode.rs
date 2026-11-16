use std::sync::Arc;
use snapfit::container;

pub struct Config;

#[container(mode = "threaded")]
pub struct InvalidModeContainer {
    #[singleton]
    config: Arc<Config>,
}
