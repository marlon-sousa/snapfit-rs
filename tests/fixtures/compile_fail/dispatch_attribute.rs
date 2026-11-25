use std::sync::Arc;
use snapfit::container;

pub trait Store {}
pub struct PostgresStore;
pub struct MockStore;

#[container]
pub struct DispatchContainer {
    #[dispatch(PostgresStore, MockStore)]
    store: Arc<dyn Store>,
}
