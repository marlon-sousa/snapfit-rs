#![deny(warnings)]
//! Chapter 7: a container whose leaves are built with `Default`, unless a field says otherwise.

use snapfit::container;

// ANCHOR: domain
#[derive(Clone, Default)]
pub struct DiscountCodes {
    pub codes: Vec<String>,
}

#[derive(Clone, Default)]
pub struct GiftWrapOptions {
    pub papers: Vec<String>,
}

#[derive(Clone, Default)]
pub struct TaxRateTable {
    pub percentage: u64,
}

impl TaxRateTable {
    pub fn new() -> Self {
        Self { percentage: 10 }
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container(leaf_build = default)]
pub struct PromotionsContainer {
    #[singleton]
    discounts: DiscountCodes,

    #[singleton]
    gift_wrap: GiftWrapOptions,

    #[singleton(build = constructor(new))]
    tax: TaxRateTable,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let promotions = PromotionsContainer::new();
    println!("discount codes: {}", promotions.get_discounts().codes.len());
    println!("gift papers: {}", promotions.get_gift_wrap().papers.len());
    println!("tax rate: {}%", promotions.get_tax().percentage);
}
// ANCHOR_END: run
