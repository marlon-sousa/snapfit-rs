//! Policy: what a container's dispatch declarations, and the fields that
//! name a declared type, may not do. Plays the role of its parent.

use std::iter;

use super::{Held, sorted};
use crate::domain::{
    ContainerDeclaration, DependencyDeclaration, DispatchExtension, DispatchedTypeDeclaration,
    DomainError, StaticDispatchProblem, TypeName, WrittenType,
};

/// Every problem of the container's dispatch declarations, and of a field
/// that names a declared type through a path.
pub(crate) fn dispatch_violations<C, F, S, T: Clone>(
    container: &ContainerDeclaration<C, F, S, T>,
) -> Vec<DomainError> {
    let sorted = sorted(&container.dispatched_types);
    let held = Held::new(&container.dependencies, &container.extensions);
    let names: Vec<_> = sorted
        .first
        .iter()
        .map(|declaration| declaration.name.clone())
        .collect();
    let repeated = sorted.repeated.iter().copied().map(repeated_name);
    let undeclared = container
        .extensions
        .iter()
        .filter(|extension| is_undeclared(extension, &names))
        .map(|extension| undeclared(extension, &names));
    let empty = sorted
        .first
        .iter()
        .copied()
        .filter(|declaration| held.variants(declaration).is_empty())
        .map(without_implementations);
    let paths = iter::repeat(&names)
        .zip(&container.dependencies)
        .filter_map(through_path);
    repeated
        .chain(undeclared)
        .chain(empty)
        .chain(paths)
        .map(DomainError::StaticDispatch)
        .collect()
}

fn repeated_name<S, T>(declaration: &DispatchedTypeDeclaration<S, T>) -> StaticDispatchProblem {
    StaticDispatchProblem::RepeatedName {
        position: declaration.position,
        name: declaration.name.clone(),
    }
}

fn is_undeclared<T>(extension: &DispatchExtension<T>, declared: &[TypeName]) -> bool {
    !declared.contains(&extension.name)
}

fn undeclared<T>(extension: &DispatchExtension<T>, declared: &[TypeName]) -> StaticDispatchProblem {
    StaticDispatchProblem::Undeclared {
        position: extension.position,
        name: extension.name.clone(),
        declared: declared.to_vec(),
    }
}

fn without_implementations<S, T>(
    declaration: &DispatchedTypeDeclaration<S, T>,
) -> StaticDispatchProblem {
    StaticDispatchProblem::WithoutImplementations {
        position: declaration.position,
        name: declaration.name.clone(),
    }
}

fn through_path<F, T>(
    (declared, dependency): (&Vec<TypeName>, &DependencyDeclaration<F, T>),
) -> Option<StaticDispatchProblem> {
    let WrittenType::Path { last, written } = &dependency.written_type else {
        return None;
    };
    declared
        .contains(last)
        .then(|| StaticDispatchProblem::ThroughPath {
            dependency: dependency.name.clone(),
            name: last.clone(),
            written: written.clone(),
        })
}

#[cfg(test)]
mod tests;
