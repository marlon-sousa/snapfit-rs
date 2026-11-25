#![deny(warnings)]
//! Chapter 13: price rules dispatched statically, through a type the container declares.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: config
pub struct ShopConfig {
    pub member_discount: u64,
}

impl ShopConfig {
    pub fn load() -> Self {
        Self {
            member_discount: 10,
        }
    }
}
// ANCHOR_END: config

// ANCHOR: trait
#[snapfit::static_dispatch]
pub trait PriceRule {
    fn price(&self, cents: u64) -> u64;
    fn describe(&self) -> String;
}
// ANCHOR_END: trait

// ANCHOR: rules
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

pub struct MemberPrice {
    percent_off: u64,
}

impl MemberPrice {
    pub fn from_config(config: Arc<ShopConfig>) -> Self {
        Self {
            percent_off: config.member_discount,
        }
    }
}

impl PriceRule for MemberPrice {
    fn price(&self, cents: u64) -> u64 {
        cents - cents * self.percent_off / 100
    }

    fn describe(&self) -> String {
        format!("member price, {} percent off", self.percent_off)
    }
}
// ANCHOR_END: rules

// ANCHOR: basket
pub struct Basket {
    prices: Arc<Prices>,
}

impl Basket {
    pub fn new(prices: Arc<Prices>) -> Self {
        Self { prices }
    }

    pub fn total(&self, items: &[u64]) -> u64 {
        items.iter().map(|&cents| self.prices.price(cents)).sum()
    }

    pub fn receipt(&self, items: &[u64]) -> String {
        format!(
            "{} items at {}: {} cents",
            items.len(),
            self.prices.describe(),
            self.total(items)
        )
    }
}
// ANCHOR_END: basket

// ANCHOR: container
#[container]
#[static_dispatch_for(PriceRule as Prices)]
#[cfg_attr(test, extend_static_dispatch(Prices, [FixedPrice]))]
pub struct ShopContainer {
    #[singleton(build = constructor(load))]
    config: Arc<ShopConfig>,

    #[singleton(implementation = ListPrice)]
    prices: Arc<Prices>,

    #[singleton(implementation = MemberPrice, deps = [config], build = constructor(from_config))]
    member_prices: Arc<Prices>,

    #[singleton(deps = [prices])]
    guest_basket: Arc<Basket>,

    #[singleton(deps = [member_prices])]
    member_basket: Arc<Basket>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();
    let items = [1000, 2500, 400];
    println!("guest: {}", shop.get_guest_basket().receipt(&items));
    println!("member: {}", shop.get_member_basket().receipt(&items));
}
// ANCHOR_END: run

// ANCHOR: test
#[cfg(test)]
pub struct FixedPrice(pub u64);

#[cfg(test)]
impl PriceRule for FixedPrice {
    fn price(&self, _cents: u64) -> u64 {
        self.0
    }

    fn describe(&self) -> String {
        format!("a fixed {} cents", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_basket_adds_up_what_the_rule_says() {
        let basket = Basket::new(Arc::new(Prices::of(FixedPrice(100))));
        assert_eq!(basket.total(&[1000, 2500, 400]), 300);
    }
}
// ANCHOR_END: test
