use crate::domain::ir::DependencyName;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("dependency cycle detected: {}", .0.iter().map(|d| d.as_str()).collect::<Vec<_>>().join(" -> "))]
    CycleDetected(Vec<DependencyName>),

    #[error("dependency '{dependency}' is missing prerequisites: {}", .missing.iter().map(|d| d.as_str()).collect::<Vec<_>>().join(", "))]
    MissingPrerequisites {
        dependency: DependencyName,
        missing: Vec<DependencyName>,
    },

    #[error("duplicate dependency name: '{0}'")]
    DuplicateDependency(DependencyName),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),
}
