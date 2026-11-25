//! Error: why a trait marked `#[snapfit::static_dispatch]` cannot be
//! dispatched statically, or why the attribute cannot be read.

use derive_more::Display;

use crate::domain::dispatched_trait::{ItemPosition, TraitItem};
use crate::domain::names::TraitName;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum DispatchProblem {
    #[display("{item} in trait '{trait_name}' {problem}")]
    Item {
        trait_name: TraitName,
        item: TraitItem,
        position: ItemPosition,
        problem: ItemProblem,
    },
    #[display(
        "trait '{_0}' is unsafe, and snapfit cannot promise that its contract holds for the type that dispatches it; remove unsafe from the trait, or dispatch it dynamically"
    )]
    UnsafeTrait(TraitName),
    #[display(
        "#[snapfit::static_dispatch] goes on a trait, and this item is not one; put it on the trait the container dispatches"
    )]
    NotATrait,
    #[display(
        "#[snapfit::static_dispatch] takes no arguments; write it alone, as #[snapfit::static_dispatch]"
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
