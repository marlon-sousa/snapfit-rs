//! Entity: what the impls of a dispatched trait forward, item by item, as
//! the emitter writes them for each of snapfit's dispatch types.

use derive_new::new;
use strum::EnumIs;

use crate::domain::declaration::Execution;
use crate::domain::dispatched_trait::Safety;

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
        /// Whether the call is awaited.
        execution: Execution,
        /// Whether the call is made in an `unsafe` block.
        safety: Safety,
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
