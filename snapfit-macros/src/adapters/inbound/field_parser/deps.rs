//! Adapter: reads the entries of `deps = [...]` into the field's
//! prerequisites, with the setter each sibling is passed to.

use strum::{EnumIs, EnumTryAs};

use super::super::darling_models::{PrerequisiteEntry, PrerequisiteList, PrerequisiteMeta};
use super::rules::attribute_error;
use crate::domain::{
    AttributeProblem, DependencyName, DomainError, DomainErrors, Prerequisite, SetterName,
};

#[cfg(test)]
mod tests;

/// The entries of one field's `deps`, each in the order written.
#[cfg_attr(test, derive(Debug))]
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
    list: PrerequisiteList,
) -> Result<Deps, DomainErrors> {
    let setters = list.0.iter().filter_map(written_setter).collect();
    let read = DomainErrors::collect(list.0.into_iter().map(|entry| read_entry(name, entry)))?;
    let (siblings, parents): (Vec<_>, Vec<_>) = read.into_iter().partition(Read::is_sibling);
    Ok(Deps {
        siblings: siblings
            .into_iter()
            .filter_map(Read::try_as_sibling)
            .collect(),
        parents: parents
            .into_iter()
            .filter_map(Read::try_as_parent)
            .collect(),
        setters,
    })
}

/// One entry of `deps`, read.
#[derive(EnumIs, EnumTryAs)]
enum Read {
    Sibling(Prerequisite),
    /// `parent(...)`.
    Parent(DependencyName),
}

/// The entry, or its problem as an error of the field `name`.
fn read_entry(name: &DependencyName, entry: PrerequisiteEntry) -> Result<Read, DomainErrors> {
    entry
        .and_then(read_meta)
        .map_err(|problem| attribute_error(name, problem).into())
}

fn read_meta(entry: PrerequisiteMeta) -> Result<Read, AttributeProblem> {
    match entry {
        PrerequisiteMeta::Parent(parent) => dependency_name(parent).map(Read::Parent),
        sibling => prerequisite(sibling).map(Read::Sibling),
    }
}

fn written_setter(entry: &PrerequisiteEntry) -> Option<syn::Ident> {
    entry.as_ref().ok().and_then(setter)
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
