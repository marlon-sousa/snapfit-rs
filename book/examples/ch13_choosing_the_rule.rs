#![deny(warnings)]
//! Chapter 13: a factory that chooses, while the program starts, which price rule a dispatched field holds.

use snapfit::container;
use std::sync::Arc;

#[snapfit::static_dispatch]
pub trait PriceRule {
    fn price(&self, cents: u64) -> u64;
    fn describe(&self) -> String;
}

#[derive(Default)]
pub struct ListPrice;

impl ListPrice {
    pub fn new() -> Self {
        Self
    }
}

impl PriceRule for ListPrice {
    fn price(&self, cents: u64) -> u64 {
        cents
    }

    fn describe(&self) -> String {
        "list price".to_string()
    }
}

// ANCHOR: sale
pub struct SalePrice {
    percent_off: u64,
}

impl SalePrice {
    pub fn new(percent_off: u64) -> Self {
        Self { percent_off }
    }
}

impl PriceRule for SalePrice {
    fn price(&self, cents: u64) -> u64 {
        cents - cents * self.percent_off / 100
    }

    fn describe(&self) -> String {
        format!("sale price, {} percent off", self.percent_off)
    }
}
// ANCHOR_END: sale

// ANCHOR: config
pub struct ShopConfig {
    pub sale_percent: Option<u64>,
}

impl ShopConfig {
    pub fn load() -> Self {
        Self { sale_percent: None }
    }
}
// ANCHOR_END: config

pub struct Basket {
    prices: Arc<Prices>,
}

impl Basket {
    pub fn new(prices: Arc<Prices>) -> Self {
        Self { prices }
    }

    pub fn receipt(&self, items: &[u64]) -> String {
        let total: u64 = items.iter().map(|&cents| self.prices.price(cents)).sum();
        format!(
            "{} items at {}: {total} cents",
            items.len(),
            self.prices.describe()
        )
    }
}

// ANCHOR: container
#[container]
#[static_dispatch_for(PriceRule as Prices, extra = [ListPrice, SalePrice])]
pub struct SaleShopContainer {
    #[singleton(build = constructor(load))]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], build = factory)]
    prices: Arc<Prices>,

    #[singleton(deps = [prices])]
    basket: Arc<Basket>,
}

impl SaleShopContainerFactories for SaleShopContainer {
    fn build_prices(inputs: SaleShopContainerPricesInputs) -> Arc<Prices> {
        let SaleShopContainerPricesInputs { config } = inputs;
        let rule = match config.sale_percent {
            Some(percent_off) => Prices::of(SalePrice::new(percent_off)),
            None => Prices::of(ListPrice::new()),
        };
        Arc::new(rule)
    }
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let items = [1000, 2500, 400];
    let shop = SaleShopContainer::new();
    println!("today: {}", shop.get_basket().receipt(&items));

    let sale_day = ShopConfig {
        sale_percent: Some(25),
    };
    let inputs = SaleShopContainerPricesInputs {
        config: Arc::new(sale_day),
    };
    let prices = SaleShopContainer::build_prices(inputs);
    println!("sale day: {}", Basket::new(prices).receipt(&items));
}
// ANCHOR_END: run
