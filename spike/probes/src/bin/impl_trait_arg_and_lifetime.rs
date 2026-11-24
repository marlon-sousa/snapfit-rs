struct A; struct B;
#[dispatchable::dispatchable]
pub trait Db<'a> { fn show(&self, x: impl std::fmt::Display, s: &'a str) -> String; }
impl<'a> Db<'a> for A { fn show(&self, x: impl std::fmt::Display, s: &'a str) -> String { format!("A {x} {s}") } }
impl<'a> Db<'a> for B { fn show(&self, x: impl std::fmt::Display, s: &'a str) -> String { format!("B {x} {s}") } }
fn main() { let d: oneof::OneOf2<A, B> = oneof::OneOf2::First(A); println!("{}", d.show(3, "x")); }
