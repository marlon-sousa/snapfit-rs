use snapfit::container;

#[derive(Clone, Default)]
pub struct Config;

#[container(crate = "snapfit")]
pub struct Shop {
    #[singleton(build = default)]
    config: Config,
}
