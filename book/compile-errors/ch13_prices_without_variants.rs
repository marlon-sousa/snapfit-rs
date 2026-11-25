use snapfit::container;
use std::sync::Arc;

#[snapfit::static_dispatch]
pub trait PriceRule {
    fn price(&self, cents: u64) -> u64;
}

pub struct ShopConfig {
    pub sale_percent: Option<u64>,
}

impl ShopConfig {
    pub fn load() -> Self {
        Self { sale_percent: None }
    }
}

#[container]
#[static_dispatch_for(PriceRule as Prices)]
pub struct SaleShopContainer {
    #[singleton(build = constructor(load))]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], build = factory)]
    prices: Arc<Prices>,
}

fn main() {}
