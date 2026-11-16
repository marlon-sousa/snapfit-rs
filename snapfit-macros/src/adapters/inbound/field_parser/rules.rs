//! Checks that refuse field declarations snapfit cannot generate code for.

use super::super::darling_models::FunctionName;
use crate::domain::{DependencyIR, DependencyName, DomainError};

pub(super) fn ensure_unconditional(
    field: &syn::Field,
    name: &DependencyName,
) -> Result<(), DomainError> {
    let is_conditional = field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"));
    (!is_conditional).then_some(()).ok_or_else(|| {
        DomainError::invalid_dependency(
            name,
            format!("field '{name}' cannot use #[cfg] or #[cfg_attr]; conditional dependencies are not supported"),
        )
    })
}

pub(super) fn ensure_constructible(dep: &DependencyIR) -> Result<(), DomainError> {
    let is_bare_trait_object = dep.trait_dispatch.is_dyn() && dep.custom_builder.is_none();
    (!is_bare_trait_object).then_some(()).ok_or_else(|| {
        DomainError::invalid_dependency(
            &dep.name,
            format!(
                "field '{}' has a trait-object type, which has no constructor; build it with a builder such as builder = build_store (unstable feature)",
                dep.name
            ),
        )
    })
}

pub(super) fn validate_function_name(
    dependency: &DependencyName,
    value: Option<FunctionName>,
    label: &str,
) -> Result<Option<String>, DomainError> {
    let problem = match value {
        None => return Ok(None),
        Some(FunctionName::Bare(name)) => return Ok(Some(name)),
        Some(FunctionName::Quoted(text)) if syn::parse_str::<syn::Ident>(&text).is_ok() => {
            format!(
                "{label} \"{text}\" on field '{dependency}' must be written without quotes: {label} = {text}"
            )
        }
        Some(FunctionName::Quoted(text) | FunctionName::Invalid(text)) => {
            format!(
                "{label} '{text}' on field '{dependency}' must be a function name, such as {label} = connect"
            )
        }
    };
    Err(DomainError::invalid_dependency(dependency, problem))
}
