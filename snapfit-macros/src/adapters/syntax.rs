//! Adapter: the user's own syntax, which the inbound adapter puts in the
//! declaration and the outbound adapter writes back out.

use crate::domain::{
    ConstructionPlan, ContainerDeclaration, DependencyDeclaration, DispatchAttribute, DispatchPlan,
};

/// The container struct, apart from its fields.
#[derive(Debug, Clone)]
pub(crate) struct ContainerSyntax {
    pub(crate) ident: syn::Ident,
    pub(crate) vis: syn::Visibility,
    pub(crate) attrs: Vec<syn::Attribute>,
    /// The function of `leaf_build = constructor(...)` as the user wrote it.
    pub(crate) leaf_constructor: Option<syn::Ident>,
}

/// One field, without snapfit's own attributes.
#[derive(Debug, Clone)]
pub(crate) struct FieldSyntax {
    pub(crate) ident: syn::Ident,
    pub(crate) ty: syn::Type,
    /// `T` when the field's type is `Arc<T>`, otherwise the field's type.
    pub(crate) arc_inner: syn::Type,
    /// The function of `build = constructor(...)` as the user wrote it.
    pub(crate) constructor: Option<syn::Ident>,
    /// The functions of `build = builder`, when the field says it.
    pub(crate) builder: Option<BuilderSyntax>,
    /// The setter each sibling entry of `deps` is passed to, in order, as the
    /// user wrote it: its label, or the entry itself.
    pub(crate) setters: Vec<syn::Ident>,
    /// The `fallible` word of `fallible = true`, where the field's error is
    /// wrapped, so that a builder that returns a plain value is reported there.
    pub(crate) fallible: Option<syn::Ident>,
    /// The type of `implementation = ...`, which the container builds for a
    /// field typed `Arc<dyn Trait>`.
    pub(crate) implementation: Option<syn::Type>,
    pub(crate) vis: syn::Visibility,
    pub(crate) attrs: Vec<syn::Attribute>,
}

/// The functions a builder chain starts and finishes with, as the user wrote
/// them in `builder(start = ..., finish = ...)`, or spelt at the `builder`
/// word for the ones left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuilderSyntax {
    pub(crate) start: syn::Ident,
    pub(crate) finish: syn::Ident,
}

/// A dispatched trait, apart from its items.
#[derive(Debug, Clone)]
pub(crate) struct TraitSyntax {
    pub(crate) ident: syn::Ident,
    pub(crate) generics: syn::Generics,
}

pub(crate) type Declaration = ContainerDeclaration<ContainerSyntax, FieldSyntax>;
pub(crate) type DispatchDeclaration =
    DispatchAttribute<TraitSyntax, syn::TraitItemFn, syn::TraitItemType>;
pub(crate) type TraitPlan = DispatchPlan<TraitSyntax, syn::TraitItemFn, syn::TraitItemType>;
pub(crate) type FieldDeclaration = DependencyDeclaration<FieldSyntax>;
pub(crate) type Plan = ConstructionPlan<ContainerSyntax, FieldSyntax>;

/// The field attributes snapfit reads, and removes from the struct it writes.
pub(crate) fn is_snapfit_attribute(attr: &syn::Attribute) -> bool {
    ["singleton", "transient", "dispatch"]
        .iter()
        .any(|name| attr.path().is_ident(name))
}

/// An attribute snapfit writes back as the user wrote it.
pub(crate) fn is_user_attribute(attr: &&syn::Attribute) -> bool {
    !is_snapfit_attribute(attr) && !is_container_attribute(attr)
}

/// `#[container]`, with or without settings.
pub(crate) fn is_container_attribute(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("container")
}
