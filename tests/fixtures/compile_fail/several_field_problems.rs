use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Database;

#[container]
pub struct BrokenContainer {
    config: Arc<Config>,

    #[singleton(constructor = "connect")]
    db: Arc<Database>,
}
