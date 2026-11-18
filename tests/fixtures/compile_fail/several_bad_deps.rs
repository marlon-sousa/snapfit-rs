use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Cache;
pub struct Mailer;
pub struct Clock;

#[container]
pub struct BrokenContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config + 1, a::b = config])]
    cache: Arc<Cache>,

    #[singleton(deps = config)]
    mailer: Arc<Mailer>,

    #[singleton(deps = [parent()], bogus)]
    clock: Arc<Clock>,
}
