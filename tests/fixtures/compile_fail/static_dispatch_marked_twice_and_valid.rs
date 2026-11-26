#[snapfit::static_dispatch]
#[snapfit::static_dispatch]
pub trait Database {
    fn find(&self, id: u32) -> u32;
}
