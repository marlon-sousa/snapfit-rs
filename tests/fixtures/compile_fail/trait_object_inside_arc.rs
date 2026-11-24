use std::sync::Arc;
use snapfit::container;

pub trait Store {}

pub struct Archive<S: ?Sized>(Box<S>);

#[container]
pub struct ArchiveContainer {
    #[singleton]
    archive: Arc<Archive<dyn Store>>,
}
