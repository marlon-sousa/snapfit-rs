//! Async containers whose fields are built another way, for the unstable suite:
//! one whose leaf_build skips its async leaf, and one whose fallible field fails.

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

pub struct Terminal;

impl Terminal {
    pub async fn connect(_region: Arc<Region>) -> Result<Self, String> {
        Err("the terminal is offline".to_string())
    }
}

#[container(mode = "async")]
pub struct KioskContainer {
    #[singleton(build = default)]
    region: Arc<Region>,

    #[singleton(deps = [region], async, fallible = true, build = constructor(connect))]
    terminal: Arc<Terminal>,
}
