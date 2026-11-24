//! What snapfit's runtime crate would hold: one generic enum per number of
//! implementations. A trait marked dispatchable implements itself for them.

#[derive(Clone, Debug)]
pub enum OneOf2<A, B> {
    First(A),
    Second(B),
}

#[derive(Clone, Debug)]
pub enum OneOf3<A, B, C> {
    First(A),
    Second(B),
    Third(C),
}
