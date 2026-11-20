use std::sync::Arc;
use snapfit::container;

pub struct ShopConfig;
pub struct Database;

#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [confog])]
    db: Arc<Database>,
}
