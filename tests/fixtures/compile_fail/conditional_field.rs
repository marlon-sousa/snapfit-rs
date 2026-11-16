use std::sync::Arc;
use snapfit::container;

#[derive(Default)]
pub struct Metrics;

#[container]
pub struct AppContainer {
    #[cfg(feature = "metrics")]
    #[singleton]
    metrics: Arc<Metrics>,
}
