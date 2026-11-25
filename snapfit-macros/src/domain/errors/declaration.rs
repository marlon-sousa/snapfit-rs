//! Error: why a container's struct, its names or its fields cannot be read
//! as a container.

use derive_more::Display;

use crate::domain::names::DependencyName;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum DeclarationProblem {
    #[display("a {label} cannot be blank")]
    BlankName { label: &'static str },
    #[display("tuple structs are not supported by #[container]; use named fields")]
    TupleStruct,
    #[display("multiple #[container] attributes are not allowed")]
    MultipleContainerAttributes,
    #[display("duplicate dependency name: '{_0}'")]
    DuplicateDependency(DependencyName),
    #[display(
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
