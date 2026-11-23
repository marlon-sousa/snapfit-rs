//! Adapter: reads the settings of a field attribute, and those of one that does not read.

use super::super::build_setting::{WrittenBuild, field_construction};
use super::super::darling_models::{DependencyAttrMeta, UnreadableAttribute};
use super::deps::{Deps, read_deps};
use super::fallible::read_fallible;
use super::implementation::read_implementation;
use super::rules::attribute_error;
use crate::domain::{DependencyName, DomainErrors};

#[cfg(test)]
mod tests;

/// What the settings say: the build, the deps, the `fallible` word and the
/// named implementation.
type Settings = (
    ((WrittenBuild, Deps), Option<syn::Ident>),
    Option<syn::Type>,
);

/// The field's `build`, `deps`, `fallible` and `implementation`, or every problem they have.
/// Each setting must be checked apart from the others: one rescued from an
/// attribute that does not read is checked alone.
pub(super) fn read_settings(
    name: &DependencyName,
    meta: DependencyAttrMeta,
) -> Result<Settings, DomainErrors> {
    let build = field_construction(name, meta.build).map_err(DomainErrors::from);
    let deps = read_deps(name, meta.deps);
    let fallible = read_fallible(name, meta.fallible).map_err(DomainErrors::from);
    let implementation = read_implementation(name, meta.implementations);
    DomainErrors::both(
        DomainErrors::both(DomainErrors::both(build, deps), fallible),
        implementation,
    )
}

/// The attribute's problem, then the problems of each setting that reads on its own.
pub(super) fn unreadable_field_attribute(
    name: &DependencyName,
    unreadable: UnreadableAttribute<DependencyAttrMeta>,
) -> DomainErrors {
    let UnreadableAttribute { problem, settings } = unreadable;
    let found = settings
        .into_iter()
        .map(|setting| read_settings(name, setting));
    let rest = found.filter_map(Result::err).flatten();
    DomainErrors::new(attribute_error(name, problem), rest.collect())
}
