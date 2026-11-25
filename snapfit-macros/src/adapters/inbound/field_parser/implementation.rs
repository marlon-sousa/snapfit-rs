//! Adapter: reads `implementation = <type>`, or `implementation = [<type>, ...]`, on a field.

use super::super::darling_models::{ImplementationEntry, ImplementationMeta};
use super::rules::attribute_error;
use crate::domain::{ArgumentProblem, DependencyName, DomainErrors};

#[cfg(test)]
mod tests;

/// What a field's `implementation` names: one type, or a list of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WrittenImplementation {
    One(syn::Type),
    /// Never empty.
    List(Vec<syn::Type>),
}

/// What the container builds for the field, or none when the field
/// leaves `implementation` out; every setting written in another shape is
/// refused, every entry of a list that is no type, an empty list, and a
/// second setting.
pub(super) fn read_implementation(
    name: &DependencyName,
    implementations: Vec<ImplementationMeta>,
) -> Result<Option<WrittenImplementation>, DomainErrors> {
    let repeated = (implementations.len() > 1).then_some(ArgumentProblem::RepeatedImplementation);
    let (written, problems): (Vec<_>, Vec<_>) = implementations
        .into_iter()
        .map(written_implementation)
        .partition(Result::is_ok);
    let errors = problems
        .into_iter()
        .filter_map(Result::err)
        .flatten()
        .chain(repeated)
        .map(|problem| attribute_error(name, problem))
        .collect();
    DomainErrors::check(errors)?;
    Ok(written.into_iter().find_map(Result::ok))
}

fn written_implementation(
    meta: ImplementationMeta,
) -> Result<WrittenImplementation, Vec<ArgumentProblem>> {
    match meta {
        ImplementationMeta::One(entry) => implementation_type(entry)
            .map(WrittenImplementation::One)
            .map_err(|problem| vec![problem]),
        ImplementationMeta::List(entries) if entries.is_empty() => {
            Err(vec![ArgumentProblem::EmptyImplementationList])
        }
        ImplementationMeta::List(entries) => listed_types(entries).map(WrittenImplementation::List),
    }
}

/// Every type of the list, or the problem of each entry that is none.
fn listed_types(entries: Vec<ImplementationEntry>) -> Result<Vec<syn::Type>, Vec<ArgumentProblem>> {
    let (types, problems): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .map(implementation_type)
        .partition(Result::is_ok);
    let problems: Vec<_> = problems.into_iter().filter_map(Result::err).collect();
    match problems.is_empty() {
        true => Ok(types.into_iter().filter_map(Result::ok).collect()),
        false => Err(problems),
    }
}

fn implementation_type(entry: ImplementationEntry) -> Result<syn::Type, ArgumentProblem> {
    match entry {
        ImplementationEntry::Type(ty) => Ok(ty),
        ImplementationEntry::Bare => Err(ArgumentProblem::BareImplementation),
        ImplementationEntry::InParentheses(written) => {
            Err(ArgumentProblem::ImplementationInParentheses(written))
        }
        ImplementationEntry::Quoted(text) => Err(ArgumentProblem::QuotedImplementation(text)),
        ImplementationEntry::NotAType(written) => {
            Err(ArgumentProblem::NotAnImplementationType(written))
        }
    }
}
