use std::sync::Arc;
use snapfit::container;

pub struct ShopConfig;
pub struct CartCheckout;

#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[transient(deps = [config])]
    checkout: CartCheckout,
}
