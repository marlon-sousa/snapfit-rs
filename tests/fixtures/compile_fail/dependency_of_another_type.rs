use std::sync::Arc;
use snapfit::container;

#[derive(Clone)]
pub struct Config;
impl Config { pub fn new() -> Self { Self } }

pub struct Clock;
impl Clock { pub fn new() -> Self { Self } }

pub struct Db;
impl Db { pub fn new(_clock: Arc<Clock>) -> Self { Self } }

#[container]
pub struct App {
    #[singleton]
    config: Config,

    #[singleton(deps = [config])]
    db: Arc<Db>,
}
