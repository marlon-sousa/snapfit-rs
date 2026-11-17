use std::sync::Arc;
use snapfit::container;

pub struct OrderService;
pub struct InventoryService;

#[container]
pub struct CyclicShopContainer {
    #[singleton(deps = [inventory])]
    orders: Arc<OrderService>,

    #[singleton(deps = [orders])]
    inventory: Arc<InventoryService>,
}
