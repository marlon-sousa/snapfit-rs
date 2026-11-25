//! Entity: what the impls of a dispatched trait forward, item by item, as
//! the emitter writes them for each of snapfit's dispatch types.

use derive_new::new;
use strum::EnumIs;

/// What becomes of the value a forwarded method returns.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rebuilding {
    /// Returned as the implementation returned it.
    None,
    /// A `Self` of the implementation, put back in the variant it came from.
    IntoVariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ForwardedItem<M, A> {
    Method {
        syntax: M,
        rebuilding: Rebuilding,
    },
    /// An associated type, which every implementation gives the same type.
    AssociatedType(A),
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DispatchPlan<T, M, A> {
    pub(crate) syntax: T,
    /// In the order the trait declares them.
    pub(crate) items: Vec<ForwardedItem<M, A>>,
}
