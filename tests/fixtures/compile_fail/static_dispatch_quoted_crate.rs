#[snapfit::static_dispatch(crate = "snapfit")]
pub trait Database {
    fn source(&self) -> &'static str;
}
