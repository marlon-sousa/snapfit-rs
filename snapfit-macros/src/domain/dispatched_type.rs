//! Entity: a type a container dispatches statically, as `static_dispatch_for`
//! declares it and `extend_static_dispatch` adds to it, and the list of
//! implementations it holds once planned.
//!
//! `S` is the syntax of a declaration, and `T` that of each type a field or a
//! list names, which the domain carries without reading.

use derive_new::new;
use strum::EnumIs;

use crate::domain::names::TypeName;

/// Where a `static_dispatch_for` or an `extend_static_dispatch` is among the
/// container's attributes of either kind, in the order they are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AttributePosition(pub(crate) usize);

/// One implementation a dispatched type may hold, named by how it is written,
/// so that `Mock` and `self::Mock` are two.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Variant<T> {
    pub(crate) name: TypeName,
    pub(crate) syntax: T,
}

/// `static_dispatch_for(<Trait> as <Name>, extra = [...])`.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DispatchedTypeDeclaration<S, T> {
    pub(crate) name: TypeName,
    pub(crate) position: AttributePosition,
    pub(crate) syntax: S,
    #[new(default)]
    pub(crate) extra: Vec<Variant<T>>,
}

/// `extend_static_dispatch(<Name>, [...])`.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DispatchExtension<T> {
    pub(crate) name: TypeName,
    pub(crate) position: AttributePosition,
    pub(crate) variants: Vec<Variant<T>>,
}

/// A dispatched type and every implementation it holds, in order, each once.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DispatchedTypePlan<S, T> {
    pub(crate) syntax: S,
    pub(crate) first: Variant<T>,
    pub(crate) rest: Vec<Variant<T>>,
}

/// How the type a field's value has, once the field's own `Arc` is set
/// aside, is written, so that a policy can tell a declared name.
#[derive(EnumIs, Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum WrittenType {
    /// A single name with no arguments, such as `Db`.
    Name(TypeName),
    /// A path of several segments whose last has no arguments, such as
    /// `crate::shop::Db`, with its last segment and the path as written.
    Path { last: TypeName, written: String },
    #[default]
    Other,
}

impl WrittenType {
    /// Whether the type is the single name given.
    pub(crate) fn is_written_as(&self, name: &TypeName) -> bool {
        matches!(self, Self::Name(written) if written == name)
    }
}

/// The alias a refused container still writes for a type it declares, so
/// that its struct, kept as written, still names it: with the
/// implementations it holds, or, when it holds none, as a placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DispatchedTypeFallback<S, T> {
    Planned(DispatchedTypePlan<S, T>),
    Unplanned(S),
}

/// Whether a field holds a type its container dispatches statically.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Dispatch {
    #[default]
    None,
    Static,
}
