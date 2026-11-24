struct A; struct B;
#[dispatchable::dispatchable]
pub trait Db { fn name(&self) -> &'static str; fn create() -> Self where Self: Sized; }
fn main() {}
