use std::sync::Arc;
use snapfit::container;

pub struct Config;

#[container]
#[container(mode = "sync")]
pub struct TwiceDeclaredContainer {
    #[singleton]
    config: Arc<Config>,
}
