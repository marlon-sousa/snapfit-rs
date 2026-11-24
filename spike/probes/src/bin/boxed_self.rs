struct A; struct B;
#[dispatchable::dispatchable]
pub trait Db { fn close(self: Box<Self>) -> u32; }
fn main() {}
