use std::sync::Arc;
use snapfit::container;

#[derive(Default)]
pub struct Config;

#[container]
pub struct Shop {
    #[singleton(fallible = "yes")]
    config: Arc<Config>,
}
