use crate::domain::ir::DependencyName;
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("dependency cycle detected: {}", join_names(.0, " -> "))]
    CycleDetected(Vec<DependencyName>),

    #[error(
        "unknown dependency '{}' on field '{dependency}'. You must choose between the dependencies the container can wire: [{}]",
        join_names(.missing, ", "),
        join_names(.available, ", ")
    )]
    MissingPrerequisites {
        dependency: DependencyName,
        missing: Vec<DependencyName>,
        available: Vec<DependencyName>,
    },

    #[error("duplicate dependency name: '{0}'")]
    DuplicateDependency(DependencyName),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error(
        "{0} is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
    )]
    RequiresUnstable(String),

    #[error("{0} is planned for a future version of snapfit and is not implemented yet")]
    NotYetImplemented(String),
}

fn join_names(names: &[DependencyName], sep: &str) -> String {
    names
        .iter()
        .map(DependencyName::as_str)
        .collect::<Vec<_>>()
        .join(sep)
}
