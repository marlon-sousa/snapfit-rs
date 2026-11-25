//! Entity: the types a trait marked `#[snapfit::static_dispatch]` is implemented
//! for, each holding one implementation of a closed list, and `of`, which picks
//! the variant from the value's type.
//!
//! A list nests: `Either<MySql, Either<Postgres, Only<Mock>>>` holds a `MySql`,
//! a `Postgres` or a `Mock`, so that one impl of the trait covers lists of any
//! length.

use std::marker::PhantomData;

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
    note = "the dispatched type holds the implementations its container's fields, `extra` list and `extend_static_dispatch` lists name"
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
