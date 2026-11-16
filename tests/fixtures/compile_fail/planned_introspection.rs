use std::sync::Arc;
use snapfit::container;

pub struct Config;

#[container(introspection = true)]
pub struct IntrospectedContainer {
    #[singleton]
    config: Arc<Config>,
}
