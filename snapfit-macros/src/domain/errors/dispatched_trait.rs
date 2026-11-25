//! Error: why a trait marked `#[snapfit::static_dispatch]` cannot be
//! dispatched statically, or why the attribute cannot be read.

use derive_more::Display;
use thiserror::Error;

use super::DispatchSubject;
use crate::domain::dispatched_trait::{ItemPosition, TraitItem};
use crate::domain::names::TraitName;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum DispatchProblem {
    #[error("{item} in trait '{trait_name}' {problem}")]
    Item {
        trait_name: TraitName,
        item: TraitItem,
        position: ItemPosition,
        problem: ItemProblem,
    },
    #[error(
        "trait '{0}' is unsafe, and snapfit cannot promise that its contract holds for the type that dispatches it; remove unsafe from the trait, or dispatch it dynamically"
    )]
    UnsafeTrait(TraitName),
    #[error(
        "trait '{0}' names Self in its parameters or bounds, such as where T: Into<Self> or where Self::Item: Clone, which snapfit cannot promise of the type that dispatches it; remove Self from them, or bound Self itself, as a supertrait or as where Self: Clone"
    )]
    SelfInTraitBounds(TraitName),
    #[error(
        "#[snapfit::static_dispatch] goes on a trait, and this item is not one; put it on the trait the container dispatches"
    )]
    NotATrait,
    #[error(
        "#[snapfit::static_dispatch] takes no arguments but crate, given once, the path to snapfit when a crate depends on it under another name; write it alone, as #[snapfit::static_dispatch], or as #[snapfit::static_dispatch(crate = di)]"
    )]
    Arguments,
}

/// What a `match` cannot forward about one item, worded after the item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub(crate) enum ItemProblem {
    #[display(
        "has no receiver, so a dispatched value has no implementation to call it on; take self, &self or &mut self, or move the function out of the trait"
    )]
    NoReceiver,
    #[display(
        "takes self through a type that static dispatch cannot forward, such as Box<Self>; take self, &self or &mut self instead"
    )]
    OtherReceiver,
    #[display(
        "could have a different value in each implementation, so static dispatch cannot forward it; make it a method instead, such as fn limit(&self) -> usize"
    )]
    AssociatedConst,
    #[display(
        "has generic parameters, which static dispatch does not forward in this version of snapfit; remove its parameters"
    )]
    GenericAssociatedType,
    #[display(
        "uses Self in its arguments or inside its return type, where two dispatched values could hold different implementations; use Self only as the receiver or as the whole return type"
    )]
    SelfElsewhere,
    #[display(
        "bounds Self by a trait other than Sized, which snapfit cannot promise of every implementation the dispatched type holds; remove the bound, or make it a supertrait of the trait"
    )]
    SelfBound,
    #[display(
        "bounds Self in its where clause by more than Sized, which the type snapfit gives it for the dispatched type cannot carry; remove the bound"
    )]
    TypeSelfBound,
    #[display(
        "names Self in its bounds, which snapfit cannot promise of the type that dispatches the trait; remove Self from its bounds"
    )]
    SelfInBounds,
    #[display(
        "is declared under #[cfg] or #[cfg_attr], which the bound that makes every implementation agree on it cannot follow; declare it in every build"
    )]
    ConditionalAssociatedType,
    #[display(
        "returns impl Trait, and each implementation would return a type of its own; return a named type instead, such as Box<dyn Iterator<Item = Order>>"
    )]
    ImplTraitReturn,
    #[display(
        "writes items that snapfit cannot see, so it cannot forward them; write the items in the trait itself"
    )]
    Macro,
}

impl DispatchProblem {
    pub(super) fn subject(&self) -> DispatchSubject {
        match self {
            Self::Item { position, .. } => DispatchSubject::Item(*position),
            Self::UnsafeTrait(_) => DispatchSubject::Safety,
            Self::SelfInTraitBounds(_) => DispatchSubject::Bounds,
            Self::Arguments => DispatchSubject::Arguments,
            Self::NotATrait => DispatchSubject::Annotated,
        }
    }
}
