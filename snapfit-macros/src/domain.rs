//! Entity: the domain's data, free of any compiler type.

mod build;
mod declaration;
mod errors;
mod names;
mod plan;

pub(crate) use build::FeatureGate;
pub(crate) use declaration::{
    Construction, Constructor, ContainerDeclaration, DependencyDeclaration, EnumDispatch,
    Execution, Introspection, Lifecycle, TypeParameters, TypeShape,
};
pub(crate) use errors::{AttributeProblem, DomainError, DomainErrors, Feature, FunctionRole};
pub(crate) use names::{ContainerName, DependencyName, FunctionName, TypeName};
pub(crate) use plan::{Call, ConstructionPlan, Step, Wrapping};
