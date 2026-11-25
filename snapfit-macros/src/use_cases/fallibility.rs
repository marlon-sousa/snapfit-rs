//! Policy: whether building the container can fail, so that its construction returns the first field's error.

use crate::domain::{ContainerDeclaration, DependencyDeclaration, Fallibility};

#[cfg(test)]
mod tests;

/// Fallible when any dependency is, whose error building the container returns.
pub(crate) fn container_fallibility<C, F, S, T>(
    container: &ContainerDeclaration<C, F, S, T>,
) -> Fallibility {
    if container.dependencies.iter().any(is_fallible) {
        Fallibility::Fallible
    } else {
        Fallibility::Infallible
    }
}

fn is_fallible<F, T>(dependency: &DependencyDeclaration<F, T>) -> bool {
    dependency.fallibility.is_fallible()
}
