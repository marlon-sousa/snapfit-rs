use snapfit::container;

#[derive(Clone)]
pub struct Config;
impl Config { pub fn new() -> Self { Self } }

#[derive(Clone)]
pub struct Cache;
impl Cache { pub fn new(_config: Config) -> Self { Self } }

#[container]
pub struct App {
    #[singleton(build)]
    config: Config,

    #[singleton(deps = [config], deps = [config], fallible = false, fallible = false)]
    cache: Cache,
}
