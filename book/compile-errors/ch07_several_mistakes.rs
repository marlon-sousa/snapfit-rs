use snapfit::container;
use std::sync::Arc;

pub struct ShopConfig;
pub struct Database;
pub struct OrderService;
pub struct StockService;

#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [confg])]
    db: Arc<Database>,

    #[singleton(deps = [stock])]
    orders: Arc<OrderService>,

    #[singleton(deps = [orders])]
    stock: Arc<StockService>,
}
