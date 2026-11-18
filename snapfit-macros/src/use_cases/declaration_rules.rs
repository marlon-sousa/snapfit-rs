//! Policy: what a declaration may not combine, whichever build of snapfit runs.

use std::collections::HashSet;

use crate::domain::{
    Construction, Constructor, ContainerDeclaration, DependencyDeclaration, DependencyName,
    DomainError, Prerequisite, SetterName,
};

#[cfg(test)]
mod tests;

/// Every rule a dependency breaks: a transient is read through a synchronous
/// getter, so it cannot be awaited; `Default::default()` takes no
/// prerequisites and cannot be awaited; only a builder has setters for a
/// label to name, and each setter takes one dependency.
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
        .chain(setter_violations(dependency))
        .collect()
}

fn setter_violations<F>(dependency: &DependencyDeclaration<F>) -> Vec<DomainError> {
    let name = &dependency.name;
    match &dependency.construction {
        Some(Construction::Builder(_)) => repeated_setters(&dependency.prerequisites)
            .into_iter()
            .map(|setter| DomainError::DuplicateSetter {
                dependency: name.clone(),
                setter,
            })
            .collect(),
        _ => dependency
            .prerequisites
            .iter()
            .filter_map(|prerequisite| label_without_builder(name, prerequisite))
            .collect(),
    }
}

/// Each setter more than one prerequisite is passed to, once, in the order
/// its second use comes.
fn repeated_setters(prerequisites: &[Prerequisite]) -> Vec<SetterName> {
    let mut seen = HashSet::new();
    let mut reported = HashSet::new();
    prerequisites
        .iter()
        .map(Prerequisite::setter)
        .filter(|setter| !seen.insert(setter.clone()) && reported.insert(setter.clone()))
        .collect()
}

fn label_without_builder(
    dependency: &DependencyName,
    prerequisite: &Prerequisite,
) -> Option<DomainError> {
    prerequisite
        .label
        .clone()
        .map(|label| DomainError::LabelWithoutBuilder {
            dependency: dependency.clone(),
            label,
            field: prerequisite.field.clone(),
        })
}
