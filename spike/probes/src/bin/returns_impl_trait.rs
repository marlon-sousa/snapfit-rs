struct A; struct B;
#[dispatchable::dispatchable]
pub trait Db { fn items(&self) -> impl Iterator<Item = u32>; }
fn main() {}
