use std::sync::Arc;
use snapfit::container;

pub struct Config;
impl Config { pub fn new() -> Self { Self } }

pub struct Server;
impl Server { pub fn new(_config: Config) -> Self { Self } }

#[container]
pub struct App {
    #[singleton]
    config: Config,

    #[singleton(deps = [config])]
    server: Arc<Server>,
}
