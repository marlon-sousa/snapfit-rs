//! Entity: the types a trait marked `#[snapfit::static_dispatch]` is implemented
//! for, each holding one implementation of a closed list, and `of`, which picks
//! the variant from the value's type.
//!
//! A list nests: `Either<MySql, Either<Postgres, Only<Mock>>>` holds a `MySql`,
//! a `Postgres` or a `Mock`, so that one impl of the trait covers lists of any
//! length.

use std::marker::PhantomData;

mod same_type;

pub use same_type::{SameType, same_type};

#[cfg(test)]
mod tests;

/// The first implementation of a list, or one of the rest.
#[derive(Clone, Debug)]
pub enum Either<First, Rest> {
    First(First),
    Rest(Rest),
}

/// The last implementation of a list.
#[derive(Clone, Debug)]
pub struct Only<Last>(pub Last);

/// Where `T` is in a list: at its head.
pub struct Here;

/// Where `T` is in a list: in its rest, at `I`.
pub struct There<I>(PhantomData<I>);

/// A dispatched type that holds `T` as one of its variants, found at the
/// index `I`, which the compiler infers.
#[diagnostic::on_unimplemented(
    message = "`{T}` is not one of the implementations this dispatched type holds",
    label = "not on the list of implementations",
    note = "the dispatched type holds the implementations its container's fields and `extend_static_dispatch` lists name"
)]
pub trait Holds<T, I> {
    fn hold(value: T) -> Self;
}

impl<First, Rest> Holds<First, Here> for Either<First, Rest> {
    fn hold(value: First) -> Self {
        Self::First(value)
    }
}

impl<First, Rest, T, I> Holds<T, There<I>> for Either<First, Rest>
where
    Rest: Holds<T, I>,
{
    fn hold(value: T) -> Self {
        Self::Rest(Rest::hold(value))
    }
}

impl<Last> Holds<Last, Here> for Only<Last> {
    fn hold(value: Last) -> Self {
        Self(value)
    }
}

/// A type that is on no list: implemented by none, so that a type passed to
/// `of` that the list does not hold is refused in words.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not one of the implementations this dispatched type holds",
    label = "not on the list of implementations",
    note = "the dispatched type holds the implementations its container's fields and `extend_static_dispatch` lists name"
)]
pub trait NotOnTheList {
    /// Whatever a value of such a type is asked to become, since there is
    /// no such value.
    fn absurd<Anything>(self) -> Anything;
}

/// A second candidate beside `Holds<Last, Here>`, which no type meets. With
/// one candidate alone, the compiler would take the value's type for `Last`
/// and report mismatched types instead of the message above.
impl<Last, T> Holds<T, There<Here>> for Only<Last>
where
    T: NotOnTheList,
{
    fn hold(value: T) -> Self {
        value.absurd()
    }
}

impl<First, Rest> Either<First, Rest> {
    /// `value` in its variant.
    pub fn of<T, I>(value: T) -> Self
    where
        Self: Holds<T, I>,
    {
        Self::hold(value)
    }
}

impl<Last> Only<Last> {
    /// `value` in its variant.
    pub fn of<T, I>(value: T) -> Self
    where
        Self: Holds<T, I>,
    {
        Self::hold(value)
    }
}
