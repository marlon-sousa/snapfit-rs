use std::sync::Arc;
use snapfit::container;

pub struct Exporter;

#[container]
pub struct Shop {
    #[singleton(build = builder(begin = make, finish = "done"))]
    exporter: Arc<Exporter>,
}
