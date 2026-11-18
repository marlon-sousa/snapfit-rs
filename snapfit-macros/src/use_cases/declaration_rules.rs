//! Policy: what a declaration may not combine, whichever build of snapfit runs.

use std::iter;

use crate::domain::{
    Construction, Constructor, ContainerDeclaration, DependencyDeclaration, DependencyName,
    DomainError, Prerequisite, SetterName,
};

#[cfg(test)]
mod tests;

use super::repeats::{Repeats, repeats};

/// Every rule a dependency breaks: a transient is read through a synchronous
/// getter, so it cannot be awaited; `Default::default()` takes no
/// prerequisites and cannot be awaited; only a builder has setters for a
/// label to name, and each setter takes one dependency; and no field is
/// passed twice to the same setter.
pub(crate) fn rule_violations<C, F>(container: &ContainerDeclaration<C, F>) -> Vec<DomainError> {
    container
        .dependencies
        .iter()
        .flat_map(violations_of)
        .collect()
}

fn violations_of<F>(dependency: &DependencyDeclaration<F>) -> Vec<DomainError> {
    let name = &dependency.name;
    let is_async_transient = dependency.lifecycle.is_transient() && dependency.execution.is_async();
    let async_transient = is_async_transient.then(|| DomainError::AsyncTransient {
        dependency: name.clone(),
    });
    let is_default =
        dependency.construction == Some(Construction::Constructor(Constructor::Default));
    let default_with_dependencies =
        (is_default && !dependency.prerequisites.is_empty()).then(|| {
            DomainError::DefaultWithDependencies {
                dependency: name.clone(),
            }
        });
    let async_default =
        (is_default && dependency.execution.is_async()).then(|| DomainError::AsyncDefault {
            dependency: name.clone(),
        });
    [async_transient, default_with_dependencies, async_default]
        .into_iter()
        .flatten()
        .chain(repeated_prerequisites(dependency))
        .chain(setter_violations(dependency))
        .collect()
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
