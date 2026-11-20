//! Policy: which fields the container's `leaf_build` builds.

use crate::domain::DependencyDeclaration;

/// A synchronous field with no `deps` and no `build` of its own: the
/// function chosen for every leaf returns at once, so it cannot be awaited.
pub(crate) fn is_leaf<F>(dependency: &DependencyDeclaration<F>) -> bool {
    dependency.construction.is_none()
        && dependency.prerequisites.is_empty()
        && dependency.parent_prerequisites.is_empty()
        && dependency.execution.is_sync()
}

#[cfg(test)]
mod tests;
