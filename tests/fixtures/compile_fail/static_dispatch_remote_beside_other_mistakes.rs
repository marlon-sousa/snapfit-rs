#[snapfit::static_dispatch(
    crate = "snapfit",
    crate = 3,
    remote = other::Database,
    inline,
)]
pub trait Database {
    fn source(&self) -> &'static str;
}
