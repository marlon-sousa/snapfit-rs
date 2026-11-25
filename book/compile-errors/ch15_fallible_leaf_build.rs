//! Chapter 15: a fallible field that the container's `leaf_build` would build.

use snapfit::container;
use std::sync::Arc;

#[derive(Default)]
pub struct DiscountCodes;

pub struct ShopConfig;

impl ShopConfig {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

#[container(leaf_build = default)]
pub struct ShopContainer {
    #[singleton]
    discounts: Arc<DiscountCodes>,

    #[singleton(fallible = true)]
    config: Arc<ShopConfig>,
}

fn main() {}
