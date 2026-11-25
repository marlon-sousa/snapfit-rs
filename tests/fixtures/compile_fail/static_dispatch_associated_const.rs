#[snapfit::static_dispatch]
pub trait Database {
    const LIMIT: usize;

    fn source(&self) -> &'static str;
}
