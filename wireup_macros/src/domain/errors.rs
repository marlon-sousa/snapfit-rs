use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("dependency cycle detected: {}", .0.join(" -> "))]
    CycleDetected(Vec<String>),

    #[error("dependency '{dependency}' is missing prerequisites: {}", .missing.join(", "))]
    MissingPrerequisites {
        dependency: String,
        missing: Vec<String>,
    },

    #[error("duplicate dependency name: '{0}'")]
    DuplicateDependency(String),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),
}
