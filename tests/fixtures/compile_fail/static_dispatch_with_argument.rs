#[snapfit::static_dispatch(inline)]
pub trait Database {
    fn source(&self) -> &'static str;
}
