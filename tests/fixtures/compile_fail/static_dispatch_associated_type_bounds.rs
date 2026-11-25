#[snapfit::static_dispatch]
pub trait Database {
    type Twin: PartialEq<Self>;

    #[cfg(not(any()))]
    type Row;

    type Pair
    where
        Self: Clone;

    type Kept
    where
        Self: 'static;

    fn find(&self, id: u32) -> Self::Row;
}
