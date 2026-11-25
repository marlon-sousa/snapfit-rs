#[snapfit::static_dispatch]
pub trait Database {
    #[allow(non_snake_case)]
    fn Rows(&self) -> usize;

    type Rows<'a>
    where
        Self: 'a;

    fn rows(&self) -> Self::Rows<'_>;
}
