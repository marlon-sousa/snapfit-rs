#![deny(warnings)]
//! Chapter 12: a checkout that takes its services as type parameters, tested with mockall's stand-ins.

use mockall::automock;
use std::sync::Arc;

#[automock]
trait Orders: Send + Sync {
    fn place(&self, item: &str, cents: u64) -> String;
}

#[automock]
trait Stock: Send + Sync {
    fn available(&self, item: &str) -> bool;
}

// ANCHOR: generic_controller
struct ExpressCheckout<O: Orders, S: Stock> {
    orders: Arc<O>,
    stock: Arc<S>,
}

impl<O: Orders, S: Stock> ExpressCheckout<O, S> {
    fn new(orders: Arc<O>, stock: Arc<S>) -> Self {
        Self { orders, stock }
    }

    fn checkout(&self, item: &str, cents: u64) -> String {
        if self.stock.available(item) {
            self.orders.place(item, cents)
        } else {
            format!("{item} is out of stock")
        }
    }
}
// ANCHOR_END: generic_controller

// ANCHOR: test_generic
fn express_checkout_places_an_item_in_stock() {
    let mut stock = MockStock::new();
    stock.expect_available().return_const(true);

    let mut orders = MockOrders::new();
    orders
        .expect_place()
        .times(1)
        .returning(|item, cents| format!("placed {item} for {cents} cents"));

    let checkout = ExpressCheckout::new(Arc::new(orders), Arc::new(stock));

    assert_eq!(
        checkout.checkout("lamp", 4500),
        "placed lamp for 4500 cents"
    );
}
// ANCHOR_END: test_generic

fn main() {
    express_checkout_places_an_item_in_stock();
    println!("express_checkout_places_an_item_in_stock passed");
}
