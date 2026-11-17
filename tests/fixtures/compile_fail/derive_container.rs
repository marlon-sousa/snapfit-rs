use std::sync::Arc;

use snapfit::Container;

pub struct Config;

#[derive(Container)]
pub struct AppContainer {
    config: Arc<Config>,
}
