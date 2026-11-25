//! Entity: the check that two spellings snapfit holds as one implementation
//! of a dispatched type, such as `Mock` and `self::Mock`, name one type.

/// A type that is `Other`: implemented only by `Other` itself, and sealed
/// so that no crate implements it for another type.
///
/// ```compile_fail,E0277
/// struct Archive;
/// struct Stranger;
///
/// impl snapfit::__private::SameType<Archive> for Stranger {}
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Other}` and `{Self}` are written as one implementation of a dispatched type, since they differ only by `self::`, but they are two different types here",
    label = "not the type its other spelling names",
    note = "inside a function, a type the function declares hides the module's type of the same name, which only `self::` reaches; name the module's type through `crate::` instead"
)]
pub trait SameType<Other: ?Sized>: super::sealed::SameType<Other> {}

impl<T: ?Sized> SameType<T> for T {}

/// Compiles only when `Written` is `Kept`, the spelling the dispatched type holds.
pub fn same_type<Written: ?Sized + SameType<Kept>, Kept: ?Sized>() {}
