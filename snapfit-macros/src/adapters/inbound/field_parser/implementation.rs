//! Adapter: reads `implementation = <type>`, or `implementations = [<type>, ...]`, on a field.

use super::super::darling_models::{
    ImplementationEntry, ImplementationKey, ImplementationMeta, ImplementationValue,
};
use super::rules::attribute_error;
use crate::domain::{ArgumentProblem, DependencyName, DomainErrors};

#[cfg(test)]
mod tests;

/// What a field names: one type with `implementation`, or a list of them with `implementations`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WrittenImplementation {
    One(syn::Type),
    List(Vec<syn::Type>),
}

/// What the container builds for the field, or the types its factory may
/// return, or none when the field leaves both settings out. Every setting
/// written in another shape is refused, every entry of a list that is no
/// type, an empty list, a list given to `implementation`, a single type
/// given to `implementations`, a second setting of either, and both.
pub(super) fn read_implementation(
    name: &DependencyName,
    implementations: Vec<ImplementationMeta>,
) -> Result<Option<WrittenImplementation>, DomainErrors> {
    let ones = implementations.iter().filter(is_one).count();
    let severals = implementations.len() - ones;
    let repeated = [
        (ones > 1).then_some(ArgumentProblem::RepeatedImplementation),
        (severals > 1).then_some(ArgumentProblem::RepeatedImplementations),
        (ones > 0 && severals > 0).then_some(ArgumentProblem::BothImplementationKeys),
    ];
    let (written, problems): (Vec<_>, Vec<_>) = implementations
        .into_iter()
        .map(written_implementation)
        .partition(Result::is_ok);
    let errors = problems
        .into_iter()
        .filter_map(Result::err)
        .flatten()
        .chain(repeated.into_iter().flatten())
        .map(|problem| attribute_error(name, problem))
        .collect();
    DomainErrors::check(errors)?;
    Ok(written.into_iter().find_map(Result::ok))
}

fn is_one(meta: &&ImplementationMeta) -> bool {
    meta.key.is_one()
}

fn written_implementation(
    meta: ImplementationMeta,
) -> Result<WrittenImplementation, Vec<ArgumentProblem>> {
    match (meta.key, meta.value) {
        (ImplementationKey::One, ImplementationValue::One(entry)) => implementation_type(entry)
            .map(WrittenImplementation::One)
            .map_err(|problem| vec![problem]),
        (ImplementationKey::One, ImplementationValue::List(_)) => {
            Err(vec![ArgumentProblem::ListInImplementation])
        }
        (ImplementationKey::Several, ImplementationValue::One(_)) => {
            Err(vec![ArgumentProblem::ImplementationsWithoutList])
        }
        (ImplementationKey::Several, ImplementationValue::List(entries)) if entries.is_empty() => {
            Err(vec![ArgumentProblem::EmptyImplementationList])
        }
        (ImplementationKey::Several, ImplementationValue::List(entries)) => {
            listed_types(entries).map(WrittenImplementation::List)
        }
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
