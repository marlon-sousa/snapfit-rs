use std::sync::Arc;
use snapfit::container;

pub struct Database;

#[container]
pub struct AsyncDependencyContainer {
    #[singleton(async, build = constructor(connect))]
    db: Arc<Database>,
}
