use std::sync::Arc;
use snapfit::container;

pub struct Database;

#[container]
pub struct Shop {
    #[singleton(build = factroy)]
    db: Arc<Database>,
}
