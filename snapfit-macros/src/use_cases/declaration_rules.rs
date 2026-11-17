//! Policy: what a declaration may not combine, whichever build of snapfit runs.

use crate::domain::{ContainerDeclaration, DependencyDeclaration, DomainError};

#[cfg(test)]
mod tests;

/// Every rule a dependency breaks: a transient is read through a synchronous
/// getter, so it cannot be awaited.
pub(crate) fn rule_violations<C, F>(container: &ContainerDeclaration<C, F>) -> Vec<DomainError> {
    container
        .dependencies
        .iter()
        .filter_map(violation_of)
        .collect()
}

fn violation_of<F>(dependency: &DependencyDeclaration<F>) -> Option<DomainError> {
    let is_async_transient = dependency.lifecycle.is_transient() && dependency.execution.is_async();
    is_async_transient.then(|| DomainError::AsyncTransient {
        dependency: dependency.name.clone(),
    })
}
