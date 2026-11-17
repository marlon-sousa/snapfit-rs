//! Adapter: the field attributes that cannot be turned into a declaration.

use super::super::darling_models::FunctionNameMeta;
use crate::domain::{
    AttributeProblem, Construction, Constructor, DependencyName, DomainError, FunctionName,
    FunctionRole,
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
        .ok_or_else(|| DomainError::ConditionalField {
            dependency: name.clone(),
        })
}

/// `constructor = ...` or `builder = ...`, at most one of them, each a bare
/// function name; neither means the type's own `new`.
pub(super) fn construction_of(
    dependency: &DependencyName,
    constructor: Option<FunctionNameMeta>,
    builder: Option<FunctionNameMeta>,
) -> Result<Construction, DomainError> {
    match (constructor, builder) {
        (None, None) => Ok(Construction::default()),
        (Some(constructor), None) => {
            function_name(dependency, constructor, FunctionRole::Constructor)
                .map(Constructor::Named)
                .map(Construction::Constructor)
        }
        (None, Some(builder)) => {
            function_name(dependency, builder, FunctionRole::Builder).map(Construction::Builder)
        }
        (Some(_), Some(_)) => Err(DomainError::ConstructorAndBuilder {
            dependency: dependency.clone(),
        }),
    }
}

fn function_name(
    dependency: &DependencyName,
    value: FunctionNameMeta,
    role: FunctionRole,
) -> Result<FunctionName, DomainError> {
    let dependency = dependency.clone();
    match value {
        FunctionNameMeta::Bare(name) => FunctionName::new(name),
        FunctionNameMeta::Quoted(written) if syn::parse_str::<syn::Ident>(&written).is_ok() => {
            Err(DomainError::QuotedFunctionName {
                dependency,
                role,
                written,
            })
        }
        FunctionNameMeta::Quoted(written) | FunctionNameMeta::Invalid(written) => {
            Err(DomainError::NotAFunctionName {
                dependency,
                role,
                written,
            })
        }
    }
}

/// A problem with one of the field's attributes, saying which field.
pub(super) fn attribute_error(name: &DependencyName, problem: AttributeProblem) -> DomainError {
    DomainError::FieldAttribute {
        dependency: name.clone(),
        problem,
    }
}
