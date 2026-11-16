use std::sync::Arc;
use wireup::container;

pub struct ShopConfig;

#[container]
pub struct ConflictingContainer {
    #[singleton]
    #[transient]
    config: Arc<ShopConfig>,
}
