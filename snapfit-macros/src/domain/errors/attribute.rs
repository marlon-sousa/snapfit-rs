//! Error: why a container's or a field's snapfit attributes cannot be read,
//! or are missing or repeated.

use thiserror::Error;

use super::ArgumentProblem;
use crate::domain::names::DependencyName;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum AttributeProblem {
    #[error("{problem}")]
    Container { problem: ArgumentProblem },
    #[error("field '{dependency}': {problem}")]
    Field {
        dependency: DependencyName,
        problem: ArgumentProblem,
    },
    #[error(
        "field '{dependency}' has no snapfit attribute; add #[singleton] to have the container build it"
    )]
    Unannotated { dependency: DependencyName },
    #[error("field '{dependency}' cannot declare multiple lifecycle attributes")]
    MultipleLifecycles { dependency: DependencyName },
}

impl AttributeProblem {
    pub(crate) fn subject(&self) -> Option<&DependencyName> {
        match self {
            Self::Field { dependency, .. }
            | Self::Unannotated { dependency }
            | Self::MultipleLifecycles { dependency } => Some(dependency),
            Self::Container { .. } => None,
        }
    }
}
