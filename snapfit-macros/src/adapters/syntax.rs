//! Adapter: the user's own syntax, which the inbound adapter puts in the
//! declaration and the outbound adapter writes back out.

use crate::domain::{ConstructionPlan, ContainerDeclaration, DependencyDeclaration};

/// The container struct, apart from its fields.
#[derive(Debug, Clone)]
pub(crate) struct ContainerSyntax {
    pub(crate) ident: syn::Ident,
    pub(crate) vis: syn::Visibility,
    pub(crate) attrs: Vec<syn::Attribute>,
}

/// One field, without snapfit's own attributes.
#[derive(Debug, Clone)]
pub(crate) struct FieldSyntax {
    pub(crate) ident: syn::Ident,
    pub(crate) ty: syn::Type,
    /// `T` when the field's type is `Arc<T>`, otherwise the field's type.
    pub(crate) arc_inner: syn::Type,
    pub(crate) vis: syn::Visibility,
    pub(crate) attrs: Vec<syn::Attribute>,
}

pub(crate) type Declaration = ContainerDeclaration<ContainerSyntax, FieldSyntax>;
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
    !is_snapfit_attribute(attr) && !attr.path().is_ident("container")
}
