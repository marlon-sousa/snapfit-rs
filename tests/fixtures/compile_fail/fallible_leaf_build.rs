use std::sync::Arc;
use snapfit::container;

pub struct Config;

impl Config {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

#[container(leaf_build = default)]
pub struct Shop {
    #[singleton(fallible = true)]
    config: Arc<Config>,
}
