//! Error: a feature this build of snapfit refuses, because it is unstable or
//! not implemented yet.

use derive_more::Display;
use thiserror::Error;

use crate::domain::names::{ContainerName, DependencyName};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum FeatureProblem {
    #[error(
        "{0} is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
    )]
    RequiresUnstable(Feature),
    #[error("{0} is planned for a future version of snapfit and is not implemented yet")]
    NotYetImplemented(Feature),
}

impl FeatureProblem {
    pub(crate) fn subject(&self) -> Option<&DependencyName> {
        match self {
            Self::RequiresUnstable(feature) | Self::NotYetImplemented(feature) => {
                feature.dependency()
            }
        }
    }
}

/// A feature this build of snapfit refuses, as its diagnostic names it.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum Feature {
    #[display("#[container(parent = ...)]")]
    ParentContainer,
    #[display("#[container(introspection = true)]")]
    Introspection,
    #[display("generic parameters on container '{container}'")]
    GenericContainer { container: ContainerName },
    #[display("parent dependency 'parent({parent})' on dependency '{dependency}'")]
    ParentDependency {
        parent: DependencyName,
        dependency: DependencyName,
    },
    #[display("#[container(mode = \"async\")]")]
    AsyncMode,
    #[display("async dependency '{dependency}'")]
    AsyncDependency { dependency: DependencyName },
    #[display("transient dependency '{dependency}'")]
    Transient { dependency: DependencyName },
    #[display("build = factory on async dependency '{dependency}'")]
    AsyncFactory { dependency: DependencyName },
    #[display("build = builder on async dependency '{dependency}'")]
    AsyncBuilder { dependency: DependencyName },
    #[display("fallible = true on transient dependency '{dependency}'")]
    FallibleTransient { dependency: DependencyName },
    #[display("#[snapfit::static_dispatch(remote = ...)]")]
    RemoteStaticDispatch,
}

impl Feature {
    pub(crate) fn dependency(&self) -> Option<&DependencyName> {
        match self {
            Self::ParentDependency { dependency, .. }
            | Self::AsyncDependency { dependency }
            | Self::Transient { dependency }
            | Self::AsyncFactory { dependency }
            | Self::AsyncBuilder { dependency }
            | Self::FallibleTransient { dependency } => Some(dependency),
            Self::ParentContainer
            | Self::Introspection
            | Self::GenericContainer { .. }
            | Self::AsyncMode
            | Self::RemoteStaticDispatch => None,
        }
    }
}
