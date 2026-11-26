#[snapfit::static_dispatch(crate)]
pub trait Database {
    fn source(&self) -> &'static str;
}
