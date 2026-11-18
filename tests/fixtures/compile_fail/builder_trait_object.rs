use std::sync::Arc;
use snapfit::container;

pub trait FileWriter: Send + Sync {}

#[container]
pub struct Site {
    #[singleton(build = builder)]
    writer: Arc<dyn FileWriter>,
}
