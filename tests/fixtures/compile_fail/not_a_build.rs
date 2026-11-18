use std::sync::Arc;
use snapfit::container;

pub struct Database;

#[container]
pub struct Shop {
    #[singleton(build = Database::connect)]
    db: Arc<Database>,
}
