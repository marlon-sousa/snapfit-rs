use std::sync::Arc;
use snapfit::container;

#[derive(Default)]
pub struct Config;

#[derive(Default)]
pub struct TaxRates;

#[container]
pub struct Shop {
    #[singleton(build = default)]
    config: Arc<Config>,

    #[singleton(deps = [config], build = default)]
    rates: Arc<TaxRates>,
}
