//! Error: why an async dependency cannot be built where it is declared.

use thiserror::Error;

use crate::domain::names::DependencyName;

/// What is wrong with an async dependency, which each variant names.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum AsyncProblem {
    #[error(
        "container with mode = \"sync\" cannot contain async dependencies: dependency '{0}' is async"
    )]
    InSyncContainer(DependencyName),
    #[error("transient dependency '{0}' cannot be async because getters are synchronous")]
    Transient(DependencyName),
    #[error(
        "async dependency '{0}' cannot be built with build = default, because Default::default() cannot be awaited; remove async, or build it with build = constructor and the name of an async function in brackets"
    )]
    BuiltByDefault(DependencyName),
}

impl AsyncProblem {
    pub(crate) fn subject(&self) -> &DependencyName {
        match self {
            Self::InSyncContainer(dependency)
            | Self::Transient(dependency)
            | Self::BuiltByDefault(dependency) => dependency,
        }
    }
}
