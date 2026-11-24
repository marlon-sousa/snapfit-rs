struct A; struct B;
#[dispatchable::dispatchable]
pub trait Db { const LIMIT: usize; fn name(&self) -> &'static str; }
fn main() {}
