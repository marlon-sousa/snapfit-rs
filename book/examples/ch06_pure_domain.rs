#![deny(warnings)]
//! Chapter 6: pure domain types work both standalone in unit tests and inside a container.

use snapfit::container;
use std::sync::Arc;

// ANCHOR: domain
#[derive(Default)]
pub struct PricingConfig {
    pub tax_basis_points: u64,
}

impl PricingConfig {
    pub fn new() -> Self {
        Self {
            tax_basis_points: 800,
        }
    }
}

pub struct InvoiceCalculator {
    config: Arc<PricingConfig>,
}

impl InvoiceCalculator {
    pub fn new(config: Arc<PricingConfig>) -> Self {
        Self { config }
    }

    pub fn total_with_tax(&self, net_cents: u64) -> u64 {
        let tax = (net_cents * self.config.tax_basis_points) / 10_000;
        net_cents + tax
    }
}
// ANCHOR_END: domain

// ANCHOR: container
#[container]
pub struct BillingContainer {
    #[singleton]
    config: Arc<PricingConfig>,

    #[singleton(deps = [config])]
    calculator: Arc<InvoiceCalculator>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let custom_config = Arc::new(PricingConfig {
        tax_basis_points: 500,
    });
    let standalone = InvoiceCalculator::new(custom_config);
    println!(
        "standalone total: {} cents",
        standalone.total_with_tax(10_000)
    );

    let container = BillingContainer::new();
    let wired = container.get_calculator();
    println!("container total: {} cents", wired.total_with_tax(10_000));
}
// ANCHOR_END: run
