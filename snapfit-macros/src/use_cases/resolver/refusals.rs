//! Policy: how a dependency declared twice, and the prerequisites no field
//! declares, are told.

use crate::domain::{DeclarationProblem, DependencyName, DomainError, GraphProblem};

pub(super) fn duplicate(dependency: DependencyName) -> DomainError {
    DomainError::Declaration(DeclarationProblem::DuplicateDependency(dependency))
}

/// `available` is every other dependency the container declares, offered
/// instead of `missing`.
pub(super) fn unknown_prerequisites(
    dependency: DependencyName,
    missing: Vec<DependencyName>,
    available: Vec<DependencyName>,
) -> DomainError {
    DomainError::Graph(GraphProblem::UnknownPrerequisites {
        dependency,
        missing,
        available,
    })
}
