//! Checks that refuse field declarations snapfit cannot generate code for.

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
                "field '{}' has a trait-object type, which has no constructor; build it with builder = \"...\" (unstable feature)",
                dep.name
            ),
        )
    })
}

pub(super) fn validate_optional_ident(
    dependency: &DependencyName,
    value: Option<String>,
    label: &str,
) -> Result<Option<String>, DomainError> {
    value
        .map(|text| {
            syn::parse_str::<syn::Ident>(&text)
                .map(|_| text.clone())
                .map_err(|_| {
                    DomainError::invalid_dependency(
                        dependency,
                        format!("{label} '{text}' on field '{dependency}' must be a function name, such as \"connect\""),
                    )
                })
        })
        .transpose()
}
