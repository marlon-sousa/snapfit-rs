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

    #[error("invalid configuration: {reason}")]
    InvalidDependency {
        dependency: DependencyName,
        reason: String,
    },

    #[error(
        "{feature} is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
    )]
    RequiresUnstable {
        feature: String,
        dependency: Option<DependencyName>,
    },

    #[error("{feature} is planned for a future version of snapfit and is not implemented yet")]
    NotYetImplemented {
        feature: String,
        dependency: Option<DependencyName>,
    },
}

impl DomainError {
    pub fn invalid_dependency(dependency: &DependencyName, reason: impl Into<String>) -> Self {
        Self::InvalidDependency {
            dependency: dependency.clone(),
            reason: reason.into(),
        }
    }

    /// The dependency the error is about, so diagnostics can point at its field.
    pub fn subject(&self) -> Option<&DependencyName> {
        match self {
            Self::CycleDetected(path) => path.first(),
            Self::MissingPrerequisites { dependency, .. }
            | Self::DuplicateDependency(dependency)
            | Self::InvalidDependency { dependency, .. } => Some(dependency),
            Self::RequiresUnstable { dependency, .. }
            | Self::NotYetImplemented { dependency, .. } => dependency.as_ref(),
            Self::InvalidConfiguration(_) => None,
        }
    }
}

fn join_names(names: &[DependencyName], sep: &str) -> String {
    names
        .iter()
        .map(DependencyName::as_str)
        .collect::<Vec<_>>()
        .join(sep)
}
