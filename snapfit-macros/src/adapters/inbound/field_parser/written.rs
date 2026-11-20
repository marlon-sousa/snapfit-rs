//! Adapter: the user's own syntax of one field, kept for the generated code.

use super::super::type_inspector::arc_inner_type;
use crate::adapters::syntax::{BuilderSyntax, FieldSyntax, is_user_attribute};

/// What the field's attribute wrote, kept for the generated code.
pub(super) struct WrittenField {
    pub(super) constructor: Option<syn::Ident>,
    pub(super) builder: Option<BuilderSyntax>,
    pub(super) setters: Vec<syn::Ident>,
    pub(super) fallible: Option<syn::Ident>,
    pub(super) implementation: Option<syn::Type>,
}

pub(super) fn field_syntax(
    field: &syn::Field,
    ident: syn::Ident,
    written: WrittenField,
) -> FieldSyntax {
    let WrittenField {
        constructor,
        builder,
        setters,
        fallible,
        implementation,
    } = written;
    FieldSyntax {
        ident,
        constructor,
        builder,
        setters,
        fallible,
        implementation,
        ty: field.ty.clone(),
        arc_inner: arc_inner_type(&field.ty).unwrap_or(&field.ty).clone(),
        vis: field.vis.clone(),
        attrs: field
            .attrs
            .iter()
            .filter(is_user_attribute)
            .cloned()
            .collect(),
    }
}
