//! Adapter: the field attributes that cannot be turned into a declaration.

use super::super::darling_models::UnreadableAttribute;
use crate::domain::{AttributeProblem, DependencyName, DomainError, DomainErrors};

pub(super) fn ensure_unconditional(
    field: &syn::Field,
    name: &DependencyName,
) -> Result<(), DomainError> {
    let is_conditional = field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"));
    (!is_conditional)
        .then_some(())
        .ok_or_else(|| DomainError::ConditionalField {
            dependency: name.clone(),
        })
}

/// A problem with one of the field's attributes, saying which field.
pub(super) fn attribute_error(name: &DependencyName, problem: AttributeProblem) -> DomainError {
    DomainError::FieldAttribute {
        dependency: name.clone(),
        problem,
    }
}

/// The attribute's problem, then each bad `deps` entry it holds, as errors of the field.
pub(super) fn unreadable_attribute(
    name: &DependencyName,
    unreadable: UnreadableAttribute,
) -> DomainErrors {
    let UnreadableAttribute { problem, bad_deps } = unreadable;
    DomainErrors::new(
        attribute_error(name, problem),
        bad_deps
            .into_iter()
            .map(|problem| attribute_error(name, problem))
            .collect(),
    )
}
