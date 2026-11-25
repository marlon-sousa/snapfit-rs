//! Entity: a trait marked `#[snapfit::static_dispatch]`, as the user declared
//! it, before snapfit checks what of it a `match` can forward.
//!
//! `T`, `M` and `A` are the syntax of the trait, of each method and of each
//! associated type, which the domain carries without reading so that the
//! emitter gets them back with each decision.

use derive_more::Display;
use derive_new::new;
use strum::EnumIs;

use crate::domain::declaration::TypeParameters;
use crate::domain::names::{ConstantName, MacroName, MethodName, TraitName, TypeName};

/// What the attribute itself was given, between its parentheses.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DispatchArguments {
    #[default]
    None,
    /// `remote = <path>`, the trait of another crate it would copy.
    Remote,
    /// Anything else, which the attribute does not take.
    Other,
}

#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Safety {
    #[default]
    Safe,
    /// `unsafe trait`, whose implementations promise a contract.
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

/// Whether `Self` appears where two values could hold different variants:
/// anywhere but the receiver, the whole return type, a `Self::` path and a
/// bound on `Self`.
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

/// A method's signature, as far as forwarding it goes.
#[derive(new, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MethodShape {
    pub(crate) receiver: Receiver,
    pub(crate) self_use: SelfUse,
    pub(crate) returns: ReturnShape,
}

/// Which item of the trait an error is about. A trait may give a method and
/// an associated type the same name, so the kind is part of it.
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
        parameters: TypeParameters,
    },
    AssociatedConst(ConstantName),
    /// A macro called among the items, whose expansion snapfit cannot see.
    Macro(MacroName),
}

/// What `#[snapfit::static_dispatch]` was written on.
#[derive(EnumIs, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Annotated<T, M, A> {
    Trait(TraitDeclaration<T, M, A>),
    /// Any other item, such as a struct or a function.
    Other,
}

/// `#[snapfit::static_dispatch]`, with what it was given and what it is on.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DispatchAttribute<T, M, A> {
    pub(crate) arguments: DispatchArguments,
    pub(crate) annotated: Annotated<T, M, A>,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct TraitDeclaration<T, M, A> {
    pub(crate) name: TraitName,
    pub(crate) syntax: T,
    #[new(default)]
    pub(crate) safety: Safety,
    /// In the order the trait declares them.
    #[new(default)]
    pub(crate) items: Vec<TraitItemDeclaration<M, A>>,
}
