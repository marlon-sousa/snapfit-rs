struct A; struct B;
#[dispatchable::dispatchable]
pub trait Db { fn sum(&self, (x, y): (u32, u32)) -> u32; }
impl Db for A { fn sum(&self, (x, y): (u32, u32)) -> u32 { x + y } }
impl Db for B { fn sum(&self, (x, y): (u32, u32)) -> u32 { x * y } }
fn main() { let d: oneof::OneOf2<A, B> = oneof::OneOf2::Second(B); println!("{}", d.sum((3, 4))); }
