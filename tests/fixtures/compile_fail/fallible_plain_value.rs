use std::sync::Arc;
use snapfit::container;

pub struct Config;

impl Config {
    pub fn new() -> Self {
        Self
    }
}

#[container]
pub struct Shop {
    #[singleton(fallible = true)]
    config: Arc<Config>,
}
