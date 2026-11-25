//! Adapter: reads `fallible = ...` on a field.

use super::super::darling_models::FallibleMeta;
use super::rules::attribute_error;
use crate::domain::{ArgumentProblem, DependencyName, DomainError};

#[cfg(test)]
mod tests;

/// The `fallible` word of `fallible = true`, or none when the field says
/// `fallible = false` or leaves the setting out.
pub(super) fn read_fallible(
    name: &DependencyName,
    fallible: Option<FallibleMeta>,
) -> Result<Option<syn::Ident>, DomainError> {
    let problem = match fallible {
        Some(FallibleMeta::True(word)) => return Ok(Some(word)),
        None | Some(FallibleMeta::False) => return Ok(None),
        Some(FallibleMeta::Bare) => ArgumentProblem::BareFallible,
        Some(FallibleMeta::InParentheses) => ArgumentProblem::FallibleInParentheses,
        Some(FallibleMeta::NotABoolean(written)) => ArgumentProblem::NotAFallibleValue(written),
    };
    Err(attribute_error(name, problem))
}
