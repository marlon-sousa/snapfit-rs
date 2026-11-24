struct A; struct B;
pub trait Named { fn label(&self) -> String; }
#[dispatchable::dispatchable]
pub trait Db: Named { fn name(&self) -> &'static str; }
fn main() {}
