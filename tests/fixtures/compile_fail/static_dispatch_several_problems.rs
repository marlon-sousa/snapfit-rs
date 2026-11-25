#[snapfit::static_dispatch(remote)]
pub trait Database {
    const LIMIT: usize;

    fn create() -> Self;

    fn source(&self) -> &'static str;

    fn orders(&self) -> impl Iterator<Item = u32>;
}
