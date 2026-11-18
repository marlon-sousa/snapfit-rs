use std::sync::Arc;
use snapfit::container;

pub struct Database;

impl Database {
    pub fn new() -> Self {
        Self
    }
}

pub struct Orders;

#[container]
pub struct Shop {
    #[singleton]
    read_db: Arc<Database>,

    #[singleton(deps = [read_db], build = factory)]
    orders: Arc<Orders>,
}

impl ShopFactories for Shop {
    fn build_orders(inputs: ShopOrdersInputs) -> Arc<Orders> {
        let ShopOrdersInputs { reed_db } = inputs;
        let _ = reed_db;
        Arc::new(Orders)
    }
}
