//! Entity: the private traits that seal `Holds`, `NotOnTheList` and
//! `SameType`, so that no crate but snapfit adds an implementation that
//! would make `of` take a type its list does not hold, or let two different
//! types pass as one.

use super::{Either, Here, Only, There};

/// `Holds<T, I>`, for exactly the lists and indexes snapfit implements it for.
pub trait Holds<T, I> {}

impl<First, Rest> Holds<First, Here> for Either<First, Rest> {}

impl<First, Rest, T, I> Holds<T, There<I>> for Either<First, Rest> where Rest: Holds<T, I> {}

impl<Last> Holds<Last, Here> for Only<Last> {}

impl<Last, T> Holds<T, There<Here>> for Only<Last> where T: NotOnTheList {}

/// Implemented by no type, as `NotOnTheList` is.
pub trait NotOnTheList {}

/// `SameType<Other>`, for `Other` itself only.
pub trait SameType<Other: ?Sized> {}

impl<T: ?Sized> SameType<T> for T {}
