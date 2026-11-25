#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;

    fn create() -> Self;
}
