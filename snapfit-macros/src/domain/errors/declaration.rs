//! Error: why a container's struct, its names or its fields cannot be read
//! as a container.

use thiserror::Error;

use crate::domain::names::DependencyName;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum DeclarationProblem {
    #[error("a {label} cannot be blank")]
    BlankName { label: &'static str },
    #[error("tuple structs are not supported by #[container]; use named fields")]
    TupleStruct,
    #[error("multiple #[container] attributes are not allowed")]
    MultipleContainerAttributes,
    #[error("duplicate dependency name: '{0}'")]
    DuplicateDependency(DependencyName),
    #[error(
        "field '{dependency}' cannot use #[cfg] or #[cfg_attr]; conditional dependencies are not supported"
    )]
    ConditionalField { dependency: DependencyName },
}

impl DeclarationProblem {
    pub(crate) fn subject(&self) -> Option<&DependencyName> {
        match self {
            Self::DuplicateDependency(dependency) | Self::ConditionalField { dependency } => {
                Some(dependency)
            }
            Self::BlankName { .. } | Self::TupleStruct | Self::MultipleContainerAttributes => None,
        }
    }
}
