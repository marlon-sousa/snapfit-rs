//! Adapter: reads the entries of `deps = [...]` into the field's
//! prerequisites, with the setter each sibling is passed to.

use super::super::darling_models::PrerequisiteMeta;
use super::rules::attribute_error;
use crate::domain::{AttributeProblem, DependencyName, DomainError, Prerequisite, SetterName};

#[cfg(test)]
mod tests;

/// The entries of one field's `deps`, each in the order written.
pub(super) struct Deps {
    pub(super) siblings: Vec<Prerequisite>,
    /// `parent(...)` entries.
    pub(super) parents: Vec<DependencyName>,
    /// The setter each sibling is passed to by a builder, with its span: its
    /// label, or the sibling's own word. A sibling that is not a word, such
    /// as `a::b`, names no field, so it is refused before any setter is used.
    pub(super) setters: Vec<syn::Ident>,
}

pub(super) fn read_deps(
    name: &DependencyName,
    entries: Vec<PrerequisiteMeta>,
) -> Result<Deps, DomainError> {
    let (siblings, parents): (Vec<_>, Vec<_>) =
        entries.into_iter().partition(is_sibling_or_labelled);
    let setters = siblings.iter().filter_map(setter).collect();
    let siblings = siblings
        .into_iter()
        .map(prerequisite)
        .collect::<Result<_, _>>()
        .map_err(|problem| attribute_error(name, problem))?;
    let parents = parents
        .into_iter()
        .map(PrerequisiteMeta::into_name)
        .map(dependency_name)
        .collect::<Result<_, _>>()
        .map_err(|problem| attribute_error(name, problem))?;
    Ok(Deps {
        siblings,
        parents,
        setters,
    })
}

fn is_sibling_or_labelled(entry: &PrerequisiteMeta) -> bool {
    !entry.is_parent()
}

fn setter(entry: &PrerequisiteMeta) -> Option<syn::Ident> {
    match entry {
        PrerequisiteMeta::Sibling { word, .. } => word.clone(),
        PrerequisiteMeta::Labelled { label, .. } => Some(label.clone()),
        PrerequisiteMeta::Parent(_) => None,
    }
}

fn prerequisite(entry: PrerequisiteMeta) -> Result<Prerequisite, AttributeProblem> {
    match entry {
        PrerequisiteMeta::Labelled { label, name } => Ok(Prerequisite {
            field: dependency_name(name)?,
            label: Some(SetterName::new(label.to_string()).map_err(blank)?),
        }),
        other => dependency_name(other.into_name()).map(Prerequisite::new),
    }
}

fn dependency_name(text: String) -> Result<DependencyName, AttributeProblem> {
    DependencyName::new(text).map_err(blank)
}

fn blank(_: DomainError) -> AttributeProblem {
    AttributeProblem::BlankName
}
