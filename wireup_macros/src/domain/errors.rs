use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    CycleDetected(Vec<String>),
    MissingPrerequisites {
        dependency: String,
        missing: Vec<String>,
    },
    DuplicateDependency(String),
    InvalidConfiguration(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CycleDetected(cycle) => {
                write!(f, "dependency cycle detected: {}", cycle.join(" -> "))
            }
            Self::MissingPrerequisites {
                dependency,
                missing,
            } => {
                write!(
                    f,
                    "dependency '{dependency}' is missing prerequisites: {}",
                    missing.join(", ")
                )
            }
            Self::DuplicateDependency(name) => {
                write!(f, "duplicate dependency name: '{name}'")
            }
            Self::InvalidConfiguration(msg) => {
                write!(f, "invalid configuration: {msg}")
            }
        }
    }
}

impl std::error::Error for DomainError {}
