use std::sync::Arc;
use wireup::container;

pub struct ShopConfig;

#[container]
pub struct TupleShopContainer(#[singleton] Arc<ShopConfig>);
