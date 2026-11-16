use std::sync::Arc;
use wireup::container;

pub struct Config;

#[container]
pub struct TupleContainer(#[singleton] Arc<Config>);
