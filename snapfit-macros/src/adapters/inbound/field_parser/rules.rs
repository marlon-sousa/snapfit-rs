//! Adapter: the field attributes that cannot be turned into a declaration.

use crate::domain::{
    ArgumentProblem, AttributeProblem, DeclarationProblem, DependencyName, DomainError,
};

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
        .ok_or_else(|| conditional_field(name))
}

fn conditional_field(name: &DependencyName) -> DomainError {
    DomainError::Declaration(DeclarationProblem::ConditionalField {
        dependency: name.clone(),
    })
}

/// A problem with one of the field's attributes, saying which field.
pub(super) fn attribute_error(name: &DependencyName, problem: ArgumentProblem) -> DomainError {
    DomainError::Attribute(AttributeProblem::Field {
        dependency: name.clone(),
        problem,
    })
}
