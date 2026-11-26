//! Entity: a trait marked `#[snapfit::static_dispatch]`, as the user declared
//! it, before snapfit checks what of it a `match` can forward.
//!
//! `T`, `M` and `A` are the syntax of the trait, of each method and of each
//! associated type, which the domain carries without reading so that the
//! emitter gets them back with each decision.

use derive_more::Display;
use derive_new::new;
use strum::EnumIs;

use crate::domain::declaration::{Execution, TypeParameters};
use crate::domain::names::{ConstantName, MacroName, MethodName, TraitName, TypeName};

mod arguments;

pub(crate) use arguments::{DispatchArguments, DispatchSetting, RuntimeValue, SettingPosition};

#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Safety {
    #[default]
    Safe,
    /// An `unsafe trait`, whose implementations promise a contract, or an
    /// `unsafe fn`, whose callers do.
    Unsafe,
}

/// How a method takes the value it is called on.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Receiver {
    /// An associated function, with no `self`.
    None,
    /// `self`, `mut self` or `self: Self`.
    Value,
    /// `&self`, `&mut self`, `self: &Self` or `self: &mut Self`.
    Reference,
    /// Any other type of `self`, such as `self: Box<Self>`.
    Other,
}

/// Whether `Self` appears where two values could hold different variants,
/// or where an implementation's type could not stand for it: anywhere but
/// the receiver, the whole return type, a `Self::` path and the bounded type
/// of a bound on `Self`.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelfUse {
    Forwardable,
    Elsewhere,
}

#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReturnShape {
    /// A type that names neither `Self` alone nor `impl Trait`.
    Plain,
    /// `Self` itself.
    ItSelf,
    /// A type that holds `impl Trait`, which each implementation would
    /// choose for itself.
    ImplTrait,
}

/// What an item's `where` clause bounds `Self` by, beyond `Sized`, which
/// every dispatch type is.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelfBound {
    /// Nothing but `Sized`.
    Sized,
    /// Lifetimes too, such as `Self: 'a`, which a dispatch type outlives
    /// whenever its implementations do.
    Lifetime,
    /// A trait, such as `Self: Clone`, which an impl cannot promise of the
    /// implementations it forwards to.
    Trait,
}

/// Whether an item exists in every build.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Presence {
    Always,
    /// Under `#[cfg]` or `#[cfg_attr]`.
    Conditional,
}

/// A method's signature, as far as forwarding it goes.
#[derive(new, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MethodShape {
    pub(crate) receiver: Receiver,
    pub(crate) self_use: SelfUse,
    pub(crate) self_bound: SelfBound,
    pub(crate) returns: ReturnShape,
    /// `async fn`, whose call is awaited.
    pub(crate) execution: Execution,
    /// `unsafe fn`, whose call is made in an `unsafe` block.
    pub(crate) safety: Safety,
}

/// An associated type, as far as forwarding it goes.
#[derive(new, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AssociatedTypeShape {
    pub(crate) parameters: TypeParameters,
    /// Whether its bounds name `Self`.
    pub(crate) self_use: SelfUse,
    /// What its `where` clause bounds `Self` by.
    pub(crate) self_bound: SelfBound,
    pub(crate) presence: Presence,
}

/// Where an item is among the trait's items, counted from 0, which the
/// diagnostic about it points at.
#[derive(new, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ItemPosition(pub(crate) usize);

/// Which item of the trait an error is about, as its message names it.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum TraitItem {
    #[display("method '{_0}'")]
    Method(MethodName),
    #[display("associated type '{_0}'")]
    AssociatedType(TypeName),
    #[display("associated const '{_0}'")]
    AssociatedConst(ConstantName),
    #[display("the call to the macro '{_0}!'")]
    Macro(MacroName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TraitItemDeclaration<M, A> {
    Method {
        name: MethodName,
        syntax: M,
        shape: MethodShape,
    },
    AssociatedType {
        name: TypeName,
        syntax: A,
        shape: AssociatedTypeShape,
    },
    AssociatedConst(ConstantName),
    /// A macro called among the items, whose expansion snapfit cannot see.
    Macro(MacroName),
}

/// What `#[snapfit::static_dispatch]` was written on.
#[derive(EnumIs, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Annotated<T, M, A> {
    Trait(TraitDeclaration<T, M, A>),
    /// A trait that another `#[snapfit::static_dispatch]` after this one
    /// marks too, which dispatches it.
    MarkedAgain(TraitName),
    /// Any other item, such as a struct or a function.
    Other,
}

/// `#[snapfit::static_dispatch]`, with what it was given and what it is on.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DispatchAttribute<T, M, A> {
    pub(crate) arguments: DispatchArguments,
    pub(crate) annotated: Annotated<T, M, A>,
}

/// An item with where the trait declares it.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlacedItem<M, A> {
    pub(crate) position: ItemPosition,
    pub(crate) declaration: TraitItemDeclaration<M, A>,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct TraitDeclaration<T, M, A> {
    pub(crate) name: TraitName,
    pub(crate) syntax: T,
    pub(crate) safety: Safety,
    /// Where its supertraits, parameters and `where` clause name `Self`.
    pub(crate) bounds: SelfUse,
    /// In the order the trait declares them.
    pub(crate) items: Vec<PlacedItem<M, A>>,
}
