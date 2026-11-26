use std::sync::Arc;
use snapfit::container;

#[derive(Clone)]
pub struct Clock;

#[derive(Clone)]
pub struct Meter;

impl Meter {
    pub fn connect(port: u16) -> Self {
        let _ = port;
        Self
    }
}

#[container(leaf_build = constructor(connect))]
pub struct App {
    #[singleton]
    clock: Clock,

    #[singleton]
    shared_clock: Arc<Clock>,

    #[singleton]
    meter: Meter,
}
