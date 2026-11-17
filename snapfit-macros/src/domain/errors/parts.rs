//! Error: the parts of an error worded by the error that holds them.

use derive_more::Display;

use crate::domain::names::{ContainerName, DependencyName, FunctionName};

/// A feature this build of snapfit refuses, as its diagnostic names it.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum Feature {
    #[display("#[container(parent = ...)]")]
    ParentContainer,
    #[display("#[container(introspection = true)]")]
    Introspection,
    #[display("generic parameters on container '{container}'")]
    GenericContainer { container: ContainerName },
    #[display("#[dispatch(...)] on dependency '{dependency}'")]
    Dispatch { dependency: DependencyName },
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
    #[display("custom builder '{builder}' on dependency '{dependency}'")]
    Builder {
        builder: FunctionName,
        dependency: DependencyName,
    },
}

impl Feature {
    pub(crate) fn dependency(&self) -> Option<&DependencyName> {
        match self {
            Self::Dispatch { dependency }
            | Self::ParentDependency { dependency, .. }
            | Self::AsyncDependency { dependency }
            | Self::Transient { dependency }
            | Self::Builder { dependency, .. } => Some(dependency),
            Self::ParentContainer
            | Self::Introspection
            | Self::GenericContainer { .. }
            | Self::AsyncMode => None,
        }
    }
}

/// Which setting names a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub(crate) enum FunctionRole {
    #[display("constructor")]
    Constructor,
    #[display("builder")]
    Builder,
}

/// What is wrong with an attribute's arguments, apart from which attribute.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum AttributeProblem {
    /// The words of darling or syn, which read the arguments.
    #[display("{_0}")]
    Unreadable(String),
    #[display("a name in the attribute cannot be blank")]
    BlankName,
    #[display("snapfit's attributes take their settings in parentheses, not after '='")]
    Assigned,
    #[display("invalid container mode '{_0}': write mode = \"sync\", or leave mode out")]
    InvalidMode(String),
    #[display("#[dispatch(...)] requires a parenthesized list of implementation types")]
    DispatchWithoutList,
    #[display("#[dispatch(...)] requires at least one implementation type")]
    DispatchWithoutImplementations,
    #[display("#[dispatch(...)] field type must contain a 'dyn Trait' trait object")]
    DispatchWithoutTraitObject,
    #[display("trait object in #[dispatch] must specify a primary trait bound")]
    DispatchWithoutTrait,
}
