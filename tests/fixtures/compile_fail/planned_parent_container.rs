use std::sync::Arc;
use snapfit::container;

pub struct Session;

#[container(parent = AppContainer)]
pub struct RequestContainer {
    #[singleton]
    session: Arc<Session>,
}
