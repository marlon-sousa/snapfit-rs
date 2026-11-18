//! An async container whose leaf_build skips its async leaf, for the unstable suite.

use std::error::Error;
use std::sync::Arc;

use snapfit::container;

#[derive(Default)]
pub struct Region {
    pub code: &'static str,
}

pub struct Warehouse {
    pub opened_by: &'static str,
}

impl Warehouse {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self { opened_by: "new" })
    }
}

#[container(mode = "async", leaf_build = default)]
pub struct DepotContainer {
    #[singleton]
    region: Arc<Region>,

    #[singleton(async)]
    warehouse: Arc<Warehouse>,
}
