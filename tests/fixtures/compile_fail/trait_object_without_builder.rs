use std::sync::Arc;
use snapfit::container;

pub trait Store: Send + Sync {}

#[container]
pub struct AppContainer {
    #[singleton]
    store: Arc<dyn Store>,
}
