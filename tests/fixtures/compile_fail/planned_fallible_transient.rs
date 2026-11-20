use std::sync::Arc;
use snapfit::container;

pub struct Request;

impl Request {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

#[container]
pub struct Shop {
    #[singleton]
    config: Arc<String>,
    #[transient(fallible = true)]
    request: Request,
}
