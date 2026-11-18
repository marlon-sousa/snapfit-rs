use std::sync::Arc;
use snapfit::container;

#[derive(Default)]
pub struct Database;

#[container(mode = "async")]
pub struct Shop {
    #[singleton(async, build = default)]
    db: Arc<Database>,
}
