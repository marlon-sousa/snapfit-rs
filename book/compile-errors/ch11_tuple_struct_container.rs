use std::sync::Arc;
use snapfit::container;

pub struct ShopConfig;

#[container]
pub struct TupleShopContainer(#[singleton] Arc<ShopConfig>);
