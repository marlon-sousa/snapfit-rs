macro_rules! dispatched {
    ($name:ident, $output:ty) => {
        #[snapfit::static_dispatch]
        pub trait $name {
            fn source(&self) -> $output;

            fn create() -> $output;
        }
    };
}

dispatched!(Database, Self);
