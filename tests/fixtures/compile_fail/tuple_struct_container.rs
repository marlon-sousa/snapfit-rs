use std::sync::Arc;
use snapfit::container;

pub struct Config;

#[container]
pub struct TupleContainer(#[singleton] Arc<Config>);
