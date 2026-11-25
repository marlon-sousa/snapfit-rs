macro_rules! sources {
    ($name:ident) => {
        fn $name(&self) -> &'static str;
    };
}

#[snapfit::static_dispatch]
pub trait Database {
    sources!(source);

    fn find(&self, id: u32) -> u32;

    sources!(origin);
}
