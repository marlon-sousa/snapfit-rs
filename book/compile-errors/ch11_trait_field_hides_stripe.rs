use snapfit::container;
use std::sync::Arc;

pub trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

#[derive(Default)]
pub struct StripeGateway;

impl StripeGateway {
    pub fn new() -> Self {
        Self
    }

    pub fn dashboard_url(&self) -> &'static str {
        "https://dashboard.stripe.com"
    }
}

impl PaymentGateway for StripeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("Stripe charged {cents} cents")
    }
}

#[container]
pub struct ShopContainer {
    #[singleton(implementation = StripeGateway)]
    payments: Arc<dyn PaymentGateway>,
}

fn main() {
    let shop = ShopContainer::new();
    println!("{}", shop.get_payments().dashboard_url());
}
