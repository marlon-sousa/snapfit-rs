use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Database;

#[container]
pub struct AppContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config], constructor = "connect()")]
    db: Arc<Database>,
}
