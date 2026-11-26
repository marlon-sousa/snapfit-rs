use snapfit::container;

#[derive(Clone)]
pub struct Config;
impl Config { pub fn new() -> Self { Self } }

#[container(crate, mode, leaf_build)]
pub struct App {
    #[singleton]
    config: Config,
}
