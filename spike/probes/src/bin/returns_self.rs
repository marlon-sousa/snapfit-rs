struct A; struct B;
#[dispatchable::dispatchable]
pub trait Db { fn fork(&self) -> Self where Self: Sized; }
fn main() {}
