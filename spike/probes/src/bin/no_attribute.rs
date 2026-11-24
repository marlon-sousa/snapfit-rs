struct A; struct B;
pub trait Db { fn name(&self) -> &'static str; }
impl Db for A { fn name(&self) -> &'static str { "a" } }
impl Db for B { fn name(&self) -> &'static str { "b" } }
type ShopDb = oneof::OneOf2<A, B>;
fn needs<D: Db>(d: D) -> &'static str { d.name() }
fn main() { let d: ShopDb = oneof::OneOf2::First(A); println!("{}", needs(d)); }
