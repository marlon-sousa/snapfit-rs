use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Mailer;

#[container]
pub struct BrokenContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps(config))]
    mailer: Arc<Mailer>,
}
