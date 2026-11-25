//! Adapter: reads whether an associated type of a dispatched trait has
//! parameters, names `Self` in its bounds, bounds `Self` in its `where`
//! clause, or exists only under a `cfg`.
//! Plays the role of its parent.

use syn::visit::Visit;

use super::self_use::{SelfFinder, self_bound};
use crate::domain::{AssociatedTypeShape, Presence, TypeParameters};

#[cfg(test)]
mod tests;

pub(super) fn associated_type_shape(associated: &syn::TraitItemType) -> AssociatedTypeShape {
    let parameters = if associated.generics.params.is_empty() {
        TypeParameters::None
    } else {
        TypeParameters::Declared
    };
    let mut finder = SelfFinder::new();
    for bound in &associated.bounds {
        finder.visit_type_param_bound(bound);
    }
    let presence = if associated.attrs.iter().any(is_conditional) {
        Presence::Conditional
    } else {
        Presence::Always
    };
    AssociatedTypeShape::new(
        parameters,
        finder.0,
        self_bound(associated.generics.where_clause.as_ref()),
        presence,
    )
}

/// `#[cfg(...)]`, or a `#[cfg_attr(...)]` that can write one; a `cfg_attr`
/// that writes only other attributes, such as `doc`, keeps the type in every build.
fn is_conditional(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("cfg") || (attr.path().is_ident("cfg_attr") && writes_cfg(attr))
}

fn writes_cfg(attr: &syn::Attribute) -> bool {
    let syn::Meta::List(list) = &attr.meta else {
        return false;
    };
    list.tokens.clone().into_iter().any(|token| {
        matches!(token, proc_macro2::TokenTree::Ident(ident) if ident == "cfg" || ident == "cfg_attr")
    })
}
