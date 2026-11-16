use std::sync::Arc;
use wireup::container;

pub struct Config;

#[container(mode = "threaded")]
pub struct InvalidModeContainer {
    #[singleton]
    config: Arc<Config>,
}
