#![deny(warnings)]
//! Chapter 2: four ways to let a service work with more than one payment provider.

use std::sync::Arc;

// ANCHOR: gateways
trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

struct StripeGateway;

impl PaymentGateway for StripeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("Stripe charged {cents} cents")
    }
}

struct FakeGateway;

impl PaymentGateway for FakeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("fake gateway pretended to charge {cents} cents")
    }
}
// ANCHOR_END: gateways

// ANCHOR: closed_list
enum Provider {
    Stripe,
    Fake,
}

struct OrderServiceWithList {
    payments: Box<dyn PaymentGateway>,
}

impl OrderServiceWithList {
    fn new(provider: Provider) -> Self {
        let payments: Box<dyn PaymentGateway> = match provider {
            Provider::Stripe => Box::new(StripeGateway),
            Provider::Fake => Box::new(FakeGateway),
        };
        Self { payments }
    }

    fn place_order(&self, cents: u64) -> String {
        self.payments.charge(cents)
    }
}
// ANCHOR_END: closed_list

// ANCHOR: generic
struct OrderService<P: PaymentGateway> {
    payments: P,
}

impl<P: PaymentGateway> OrderService<P> {
    fn new(payments: P) -> Self {
        Self { payments }
    }

    fn place_order(&self, cents: u64) -> String {
        self.payments.charge(cents)
    }
}

struct CheckoutController<P: PaymentGateway> {
    orders: OrderService<P>,
}

impl<P: PaymentGateway> CheckoutController<P> {
    fn new(orders: OrderService<P>) -> Self {
        Self { orders }
    }

    fn checkout(&self, cents: u64) -> String {
        format!("checkout: {}", self.orders.place_order(cents))
    }
}
// ANCHOR_END: generic

// ANCHOR: trait_object
struct SharedOrderService {
    payments: Arc<dyn PaymentGateway>,
}

impl SharedOrderService {
    fn new(payments: Arc<dyn PaymentGateway>) -> Self {
        Self { payments }
    }

    fn place_order(&self, cents: u64) -> String {
        self.payments.charge(cents)
    }
}
// ANCHOR_END: trait_object

// ANCHOR: enum
enum Gateway {
    Stripe(StripeGateway),
    Fake(FakeGateway),
}

impl PaymentGateway for Gateway {
    fn charge(&self, cents: u64) -> String {
        match self {
            Gateway::Stripe(stripe) => stripe.charge(cents),
            Gateway::Fake(fake) => fake.charge(cents),
        }
    }
}

struct OrderServiceWithEnum {
    payments: Gateway,
}

impl OrderServiceWithEnum {
    fn new(payments: Gateway) -> Self {
        Self { payments }
    }

    fn place_order(&self, cents: u64) -> String {
        self.payments.charge(cents)
    }
}
// ANCHOR_END: enum

// ANCHOR: run
fn main() {
    let listed_production = OrderServiceWithList::new(Provider::Stripe);
    let listed_test = OrderServiceWithList::new(Provider::Fake);
    println!(
        "closed list, production: {}",
        listed_production.place_order(1000)
    );
    println!("closed list, test: {}", listed_test.place_order(1000));

    let production = CheckoutController::new(OrderService::new(StripeGateway));
    let test = CheckoutController::new(OrderService::new(FakeGateway));
    println!("generic, production: {}", production.checkout(2000));
    println!("generic, test: {}", test.checkout(2000));

    let shared = SharedOrderService::new(Arc::new(StripeGateway));
    println!("trait object: {}", shared.place_order(3000));

    let enum_production = OrderServiceWithEnum::new(Gateway::Stripe(StripeGateway));
    let enum_test = OrderServiceWithEnum::new(Gateway::Fake(FakeGateway));
    println!("enum, production: {}", enum_production.place_order(4000));
    println!("enum, test: {}", enum_test.place_order(4000));
}
// ANCHOR_END: run
