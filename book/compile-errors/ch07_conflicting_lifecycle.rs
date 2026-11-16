use std::sync::Arc;
use snapfit::container;

pub struct ShopConfig;

#[container]
pub struct ConflictingContainer {
    #[singleton]
    #[transient]
    config: Arc<ShopConfig>,
}
