#![deny(warnings)]
//! Chapter 14: one container that dispatches some traits statically and others dynamically.

use snapfit::container;
use std::fmt::Display;
use std::sync::Arc;

// ANCHOR: payments
pub trait PaymentGateway: Send + Sync {
    fn charge(&self, cents: u64) -> String;
}

#[derive(Default)]
pub struct StripeGateway;

impl StripeGateway {
    pub fn new() -> Self {
        Self
    }
}

impl PaymentGateway for StripeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("Stripe charged {cents} cents")
    }
}
// ANCHOR_END: payments

// ANCHOR: prices
#[snapfit::static_dispatch]
pub trait PriceRule {
    fn price(&self, cents: u64) -> u64;
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
}
// ANCHOR_END: prices

// ANCHOR: receipts
#[snapfit::static_dispatch]
pub trait ReceiptFormat {
    fn line<L: Display>(&self, label: L, cents: u64) -> String;
}

#[derive(Default)]
pub struct PlainReceipt;

impl PlainReceipt {
    pub fn new() -> Self {
        Self
    }
}

impl ReceiptFormat for PlainReceipt {
    fn line<L: Display>(&self, label: L, cents: u64) -> String {
        format!("{label}: {cents} cents")
    }
}

#[derive(Default)]
pub struct ShoutingReceipt;

impl ShoutingReceipt {
    pub fn new() -> Self {
        Self
    }
}

impl ReceiptFormat for ShoutingReceipt {
    fn line<L: Display>(&self, label: L, cents: u64) -> String {
        format!("{}: {cents} CENTS!", label.to_string().to_uppercase())
    }
}
// ANCHOR_END: receipts

// ANCHOR: checkout
pub struct Checkout<P: PriceRule> {
    prices: Arc<P>,
    payments: Arc<dyn PaymentGateway>,
}

impl<P: PriceRule> Checkout<P> {
    pub fn new(prices: Arc<P>, payments: Arc<dyn PaymentGateway>) -> Self {
        Self { prices, payments }
    }

    pub fn pay(&self, items: &[u64]) -> String {
        let total: u64 = items.iter().map(|&cents| self.prices.price(cents)).sum();
        self.payments.charge(total)
    }
}
// ANCHOR_END: checkout

// ANCHOR: container
#[container]
#[static_dispatch_for(PriceRule as Prices)]
#[static_dispatch_for(ReceiptFormat as Receipts)]
pub struct ShopContainer {
    #[singleton(implementation = StripeGateway)]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(implementation = ListPrice)]
    prices: Arc<Prices>,

    #[singleton(implementation = PlainReceipt)]
    receipt: Arc<Receipts>,

    #[singleton(implementation = ShoutingReceipt)]
    loud_receipt: Arc<Receipts>,

    #[singleton(deps = [prices, payments])]
    checkout: Arc<Checkout<Prices>>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let shop = ShopContainer::new();
    let items = [1000, 2500, 400];
    println!("{}", shop.get_checkout().pay(&items));
    println!("{}", shop.get_receipt().line("total", 3900));
    println!("{}", shop.get_loud_receipt().line("total", 3900));
    println!("{}", shop.get_receipt().line(String::from("lamp"), 1000));
}
// ANCHOR_END: run

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    struct FixedPrice(u64);

    impl PriceRule for FixedPrice {
        fn price(&self, _cents: u64) -> u64 {
            self.0
        }
    }

    struct FakeGateway;

    impl PaymentGateway for FakeGateway {
        fn charge(&self, cents: u64) -> String {
            format!("pretended to charge {cents} cents")
        }
    }

    #[test]
    fn the_checkout_charges_what_the_rule_adds_up_to() {
        let checkout = Checkout::new(Arc::new(FixedPrice(100)), Arc::new(FakeGateway));
        assert_eq!(checkout.pay(&[1000, 2500]), "pretended to charge 200 cents");
    }
}
// ANCHOR_END: test
