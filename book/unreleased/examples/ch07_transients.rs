#![deny(warnings)]
//! Chapter 7: transient dependencies are stripped from struct storage and created on each get.

use std::mem::size_of;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use snapfit::container;

static NEXT_CART_ID: AtomicU32 = AtomicU32::new(1);

// ANCHOR: domain
#[derive(Default)]
pub struct ShopConfig {
    pub store_name: &'static str,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self {
            store_name: "Northwind Books",
        }
    }
}

pub struct CartCheckout {
    cart_id: u32,
    config: Arc<ShopConfig>,
}

impl CartCheckout {
    pub fn new(config: Arc<ShopConfig>) -> Self {
        Self {
            cart_id: NEXT_CART_ID.fetch_add(1, Ordering::Relaxed),
            config,
        }
    }

    pub fn summary(&self) -> String {
        format!("cart #{} at {}", self.cart_id, self.config.store_name)
    }

    pub fn config_ref(&self) -> Arc<ShopConfig> {
        Arc::clone(&self.config)
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[transient(deps = [config])]
    checkout: CartCheckout,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    let first = container.get_checkout();
    let second = container.get_checkout();

    println!("{}", first.summary());
    println!("{}", second.summary());
    println!(
        "shared config singleton: {}",
        Arc::ptr_eq(&first.config_ref(), &second.config_ref())
    );
    println!(
        "transient occupies zero container bytes: {}",
        size_of::<ShopContainer>() == size_of::<Arc<ShopConfig>>()
    );
}
// ANCHOR_END: run
