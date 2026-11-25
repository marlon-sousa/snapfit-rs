//! Adapter: reads whether an associated type of a dispatched trait has
//! parameters, names `Self` in its bounds, bounds `Self` in its `where`
//! clause, or exists only under a `cfg`.
//! Plays the role of its parent.

use syn::visit::Visit;

use super::self_use::{SelfFinder, self_bound};
use crate::adapters::syntax::is_conditional;
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
