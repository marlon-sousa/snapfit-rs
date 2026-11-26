use snapfit::container;

#[derive(Clone, Default)]
pub struct Config;

#[container(crate = snapfit::<u32>(), mode = "threaded")]
pub struct Shop {
    #[singleton(build = default)]
    config: Config,
}
