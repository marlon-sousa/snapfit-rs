use std::sync::Arc;
use snapfit::container;

#[derive(Default)]
pub struct TaxRates;

#[container]
pub struct Shop {
    #[singleton(fallible = true, build = default)]
    rates: Arc<TaxRates>,
}
