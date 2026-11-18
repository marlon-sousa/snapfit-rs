//! Policy: what a declaration may not combine, whichever build of snapfit runs.

use crate::domain::{
    Construction, Constructor, ContainerDeclaration, DependencyDeclaration, DomainError,
};

#[cfg(test)]
mod tests;

/// Every rule a dependency breaks: a transient is read through a synchronous
/// getter, so it cannot be awaited, and `Default::default()` takes no
/// prerequisites.
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
    [async_transient, default_with_dependencies]
        .into_iter()
        .flatten()
        .collect()
}
