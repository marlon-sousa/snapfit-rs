use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Cart;

#[container]
pub struct TransientContainer {
    #[singleton]
    config: Arc<Config>,

    #[transient(deps = [config])]
    cart: Cart,
}
