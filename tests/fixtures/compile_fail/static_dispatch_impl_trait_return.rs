pub struct Order;

#[snapfit::static_dispatch]
pub trait Database {
    fn orders(&self) -> impl Iterator<Item = Order>;
}
