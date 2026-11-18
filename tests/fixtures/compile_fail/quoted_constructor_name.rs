use std::sync::Arc;
use snapfit::container;

pub struct Database;

#[container]
pub struct AppContainer {
    #[singleton(build = constructor("connect"))]
    db: Arc<Database>,
}
