use std::sync::Arc;
use snapfit::container;

pub struct Database;

#[container(mode = "async")]
pub struct Shop {
    #[singleton(async, build = factory)]
    db: Arc<Database>,
}
