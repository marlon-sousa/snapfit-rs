use snapfit::container;

#[derive(Clone)]
pub struct Config;
impl Config { pub fn new() -> Self { Self } }

#[container(crate = snapfit, mode = "sync", leaf_build = default, crate = snapfit, mode = "threaded", leaf_build = default, mode = 3)]
pub struct App {
    #[singleton]
    config: Config,
}
