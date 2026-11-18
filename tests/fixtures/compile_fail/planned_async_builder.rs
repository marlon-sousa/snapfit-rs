use std::sync::Arc;
use snapfit::container;

pub struct Exporter;

#[container(mode = "async")]
pub struct Shop {
    #[singleton(async, build = builder)]
    exporter: Arc<Exporter>,
}
