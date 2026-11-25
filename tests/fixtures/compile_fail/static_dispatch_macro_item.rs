macro_rules! sources {
    () => {
        fn source(&self) -> &'static str;
    };
}

#[snapfit::static_dispatch]
pub trait Database {
    fn find(&self, id: u32) -> u32;

    sources!();
}
