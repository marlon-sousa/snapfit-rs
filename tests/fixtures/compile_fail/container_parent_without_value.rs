use snapfit::container;

#[derive(Clone)]
pub struct Config;
impl Config { pub fn new() -> Self { Self } }

#[container(parent)]
pub struct App {
    #[singleton]
    config: Config,
}
