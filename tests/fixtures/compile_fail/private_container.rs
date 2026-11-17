mod wiring {
    use std::sync::Arc;
    use snapfit::container;

    pub struct Config;

    impl Config {
        pub fn new() -> Self {
            Self
        }
    }

    #[container]
    struct HiddenContainer {
        #[singleton]
        config: Arc<Config>,
    }
}

pub fn build_hidden_container() {
    let _ = wiring::HiddenContainer::new();
}
