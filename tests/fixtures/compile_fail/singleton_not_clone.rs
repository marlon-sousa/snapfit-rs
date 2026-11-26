use snapfit::container;

pub struct Config;
impl Config { pub fn new() -> Self { Self } }

#[container]
pub struct App {
    #[singleton]
    config: Config,
}
