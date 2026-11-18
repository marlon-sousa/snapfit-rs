use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Database;

#[container]
pub struct BrokenContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [zeta, config, alpha])]
    db: Arc<Database>,
}
