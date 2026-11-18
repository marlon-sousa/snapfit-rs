use std::sync::Arc;
use snapfit::container;

pub struct Database;

#[container]
pub struct Shop {
    #[singleton(constructor = connect)]
    db: Arc<Database>,
}
