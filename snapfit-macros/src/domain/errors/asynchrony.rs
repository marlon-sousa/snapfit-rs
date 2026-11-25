//! Error: why an async dependency cannot be built where it is declared.

use derive_more::Display;

use crate::domain::names::DependencyName;

/// What is wrong with an async dependency, which each variant names.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum AsyncProblem {
    #[display(
        "container with mode = \"sync\" cannot contain async dependencies: dependency '{_0}' is async"
    )]
    InSyncContainer(DependencyName),
    #[display("transient dependency '{_0}' cannot be async because getters are synchronous")]
    Transient(DependencyName),
    #[display(
        "async dependency '{_0}' cannot be built with build = default, because Default::default() cannot be awaited; remove async, or build it with build = constructor and the name of an async function in brackets"
    )]
    BuiltByDefault(DependencyName),
}

impl AsyncProblem {
    pub(crate) fn dependency(&self) -> &DependencyName {
        match self {
            Self::InSyncContainer(dependency)
            | Self::Transient(dependency)
            | Self::BuiltByDefault(dependency) => dependency,
        }
    }
}
