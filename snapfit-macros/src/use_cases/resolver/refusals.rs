//! Policy: how a dependency declared twice, and the prerequisites no field
//! declares, are told.

use crate::domain::{DeclarationProblem, DependencyName, DomainError, GraphProblem};

pub(super) fn duplicate(dependency: DependencyName) -> DomainError {
    DeclarationProblem::DuplicateDependency(dependency).into()
}

/// `available` is every other dependency the container declares, offered
/// instead of `missing`.
pub(super) fn unknown_prerequisites(
    dependency: DependencyName,
    missing: Vec<DependencyName>,
    available: Vec<DependencyName>,
) -> DomainError {
    GraphProblem::UnknownPrerequisites {
        dependency,
        missing,
        available,
    }
    .into()
}
