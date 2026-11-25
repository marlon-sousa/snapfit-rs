#[snapfit::static_dispatch]
pub trait Database {
    fn find(&self, id: u32) -> u32;

    fn copied(&self) -> u32
    where
        Self: Clone;
}
