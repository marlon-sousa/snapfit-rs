//! Adapter: reads `implementation = <type>` on a field.

use super::super::darling_models::ImplementationMeta;
use super::rules::attribute_error;
use crate::domain::{ArgumentProblem, DependencyName, DomainErrors};

#[cfg(test)]
mod tests;

/// The type the container builds for the field, or none when the field
/// leaves `implementation` out; every setting written in another shape is
/// refused, and so is a second one.
pub(super) fn read_implementation(
    name: &DependencyName,
    implementations: Vec<ImplementationMeta>,
) -> Result<Option<syn::Type>, DomainErrors> {
    let repeated = (implementations.len() > 1).then_some(ArgumentProblem::RepeatedImplementation);
    let (types, problems): (Vec<_>, Vec<_>) = implementations
        .into_iter()
        .map(implementation_type)
        .partition(Result::is_ok);
    let errors = problems
        .into_iter()
        .filter_map(Result::err)
        .chain(repeated)
        .map(|problem| attribute_error(name, problem))
        .collect();
    DomainErrors::check(errors)?;
    Ok(types.into_iter().find_map(Result::ok))
}

fn implementation_type(meta: ImplementationMeta) -> Result<syn::Type, ArgumentProblem> {
    match meta {
        ImplementationMeta::Type(ty) => Ok(ty),
        ImplementationMeta::Bare => Err(ArgumentProblem::BareImplementation),
        ImplementationMeta::InParentheses(written) => {
            Err(ArgumentProblem::ImplementationInParentheses(written))
        }
        ImplementationMeta::Quoted(text) => Err(ArgumentProblem::QuotedImplementation(text)),
        ImplementationMeta::NotAType(written) => {
            Err(ArgumentProblem::NotAnImplementationType(written))
        }
    }
}
