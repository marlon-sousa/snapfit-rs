#[snapfit::static_dispatch]
pub trait Database {
    fn merge(&self, other: Self) -> usize
    where
        Self: Sized;

    fn replica(&self) -> Option<Self>
    where
        Self: Sized;

    fn rows(&self) -> Vec<Self::Row>;

    type Row;
}
