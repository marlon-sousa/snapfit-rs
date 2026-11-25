use std::pin::Pin;

#[snapfit::static_dispatch]
pub trait Database {
    fn close(self: Box<Self>);

    fn poll(self: Pin<&mut Self>) -> bool;
}
