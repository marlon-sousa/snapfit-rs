use std::sync::Arc;
use snapfit::container;

pub struct Config;

impl Config {
    pub fn load() -> Result<Self, String> {
        Ok(Self)
    }
}

#[container]
pub struct Shop {
    #[singleton(fallible = true, build = constructor(load))]
    config: Arc<Config>,
}

pub fn open() -> Shop {
    Shop::new()
}
