struct A; struct B;
#[dispatchable::dispatchable]
pub trait Named { fn label(&self) -> String; }
#[dispatchable::dispatchable]
pub trait Db: Named { fn name(&self) -> &'static str; }
impl Named for A { fn label(&self) -> String { "A".into() } }
impl Named for B { fn label(&self) -> String { "B".into() } }
impl Db for A { fn name(&self) -> &'static str { "a" } }
impl Db for B { fn name(&self) -> &'static str { "b" } }
fn main() { let d: oneof::OneOf2<A, B> = oneof::OneOf2::Second(B); println!("{} {}", d.name(), d.label()); }
