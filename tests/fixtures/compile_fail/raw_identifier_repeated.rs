use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Database;

#[container]
pub struct BrokenContainer {
    #[singleton]
    r#config: Arc<Config>,

    #[singleton(deps = [config, r#config])]
    db: Arc<Database>,
}
