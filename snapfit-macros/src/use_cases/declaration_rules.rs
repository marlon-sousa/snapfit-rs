//! Policy: what a declaration may not combine, whichever build of snapfit runs.

use std::iter;

use crate::domain::{
    AsyncProblem, Construction, Constructor, ContainerDeclaration, DependencyDeclaration,
    DependencyName, DomainError, InfallibleBuild, Prerequisite, SetterName,
};

#[cfg(test)]
mod tests;

use super::leaf::is_leaf;
use super::repeats::{Repeats, repeats};

/// Every rule a dependency breaks: a transient is read through a synchronous
/// getter, so it cannot be awaited; `Default::default()` takes no
/// prerequisites, cannot be awaited and cannot fail, and neither can the
/// function `leaf_build` names; only a builder has setters for a label to
/// name, and each setter takes one dependency; and no field is passed twice
/// to the same setter.
pub(crate) fn rule_violations<C, F>(container: &ContainerDeclaration<C, F>) -> Vec<DomainError> {
    let leaves = iter::repeat(&container.leaf_constructor).zip(&container.dependencies);
    container
        .dependencies
        .iter()
        .flat_map(violations_of)
        .chain(leaves.filter_map(fallible_leaf))
        .collect()
}

/// A fallible field that a `leaf_build` other than `new` would build. Left
/// to its own `new` instead, the container would say one thing and do
/// another.
fn fallible_leaf<F>(
    (leaf, dependency): (&Constructor, &DependencyDeclaration<F>),
) -> Option<DomainError> {
    (!leaf.is_new() && is_leaf(dependency) && dependency.fallibility.is_fallible())
        .then(|| cannot_fail(&dependency.name, InfallibleBuild::LeafBuild))
}

fn violations_of<F>(dependency: &DependencyDeclaration<F>) -> Vec<DomainError> {
    let name = &dependency.name;
    let is_async_transient = dependency.lifecycle.is_transient() && dependency.execution.is_async();
    let async_transient = is_async_transient.then(|| async_transient(name));
    let is_default =
        dependency.construction == Some(Construction::Constructor(Constructor::Default));
    let default_with_dependencies = (is_default && !dependency.prerequisites.is_empty())
        .then(|| default_with_dependencies(name));
    let async_default =
        (is_default && dependency.execution.is_async()).then(|| async_default(name));
    let fallible_default = (is_default && dependency.fallibility.is_fallible())
        .then(|| cannot_fail(name, InfallibleBuild::Default));
    [
        async_transient,
        default_with_dependencies,
        async_default,
        fallible_default,
    ]
    .into_iter()
    .flatten()
    .chain(repeated_prerequisites(dependency))
    .chain(setter_violations(dependency))
    .collect()
}

fn async_transient(dependency: &DependencyName) -> DomainError {
    DomainError::Async(AsyncProblem::Transient(dependency.clone()))
}

fn default_with_dependencies(dependency: &DependencyName) -> DomainError {
    DomainError::DefaultWithDependencies {
        dependency: dependency.clone(),
    }
}

fn async_default(dependency: &DependencyName) -> DomainError {
    DomainError::Async(AsyncProblem::BuiltByDefault(dependency.clone()))
}

fn cannot_fail(dependency: &DependencyName, build: InfallibleBuild) -> DomainError {
    DomainError::BuildCannotFail {
        dependency: dependency.clone(),
        build,
    }
}

fn repeated_prerequisites<F>(dependency: &DependencyDeclaration<F>) -> Vec<DomainError> {
    iter::repeat(&dependency.name)
        .zip(uses(&dependency.prerequisites).repeated)
        .map(repeated_prerequisite)
        .collect()
}

/// Each prerequisite as the field it passes and the setter that takes it.
fn uses(prerequisites: &[Prerequisite]) -> Repeats<(&DependencyName, SetterName)> {
    repeats(prerequisites.iter().map(field_and_setter))
}

fn repeated_prerequisite(
    (dependency, (prerequisite, _)): (&DependencyName, (&DependencyName, SetterName)),
) -> DomainError {
    DomainError::RepeatedPrerequisite {
        dependency: dependency.clone(),
        prerequisite: prerequisite.clone(),
    }
}

fn setter_violations<F>(dependency: &DependencyDeclaration<F>) -> Vec<DomainError> {
    let name = iter::repeat(&dependency.name);
    match &dependency.construction {
        Some(Construction::Builder(_)) => name
            .zip(repeated_setters(&dependency.prerequisites))
            .map(duplicate_setter)
            .collect(),
        _ => name
            .zip(&dependency.prerequisites)
            .filter_map(label_without_builder)
            .collect(),
    }
}

/// Each setter more than one field is passed to, once, in the order its
/// second use comes. A field passed twice to the same setter is reported as
/// a repeated prerequisite, not as a second use of its setter.
fn repeated_setters(prerequisites: &[Prerequisite]) -> Vec<SetterName> {
    repeats(uses(prerequisites).distinct.into_iter().map(setter_of)).repeated
}

fn field_and_setter(prerequisite: &Prerequisite) -> (&DependencyName, SetterName) {
    (&prerequisite.field, prerequisite.setter())
}

fn setter_of((_, setter): (&DependencyName, SetterName)) -> SetterName {
    setter
}

fn duplicate_setter((dependency, setter): (&DependencyName, SetterName)) -> DomainError {
    DomainError::DuplicateSetter {
        dependency: dependency.clone(),
        setter,
    }
}

fn label_without_builder(
    (dependency, prerequisite): (&DependencyName, &Prerequisite),
) -> Option<DomainError> {
    let label = prerequisite.label.clone()?;
    Some(DomainError::LabelWithoutBuilder {
        dependency: dependency.clone(),
        label,
        field: prerequisite.field.clone(),
    })
}
