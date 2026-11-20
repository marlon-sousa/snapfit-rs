use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Settings;

impl Config {
    pub fn load() -> Result<Settings, String> {
        Ok(Settings)
    }
}

#[container]
pub struct Shop {
    #[singleton(fallible = true, build = constructor(load))]
    config: Arc<Config>,
}
