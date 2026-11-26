use snapfit::static_dispatch;

#[snapfit::static_dispatch]
#[doc = "Where the orders are kept."]
#[static_dispatch]
#[cfg_attr(all(), snapfit::static_dispatch)]
pub trait Database {
    fn find(&self, id: u32) -> u32;
    fn create() -> Self;
}
