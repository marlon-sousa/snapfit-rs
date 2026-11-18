//! Entity: the domain's data, free of any compiler type.

mod build;
mod declaration;
mod errors;
mod names;
mod plan;

pub(crate) use build::FeatureGate;
pub(crate) use declaration::{
    BuilderMethods, Construction, Constructor, ContainerDeclaration, DependencyDeclaration,
    EnumDispatch, Execution, Introspection, Lifecycle, Prerequisite, TypeParameters, TypeShape,
};
pub(crate) use errors::{AttributeProblem, DomainError, DomainErrors, Feature};
pub(crate) use names::{ContainerName, DependencyName, FunctionName, SetterName, TypeName};
pub(crate) use plan::{Call, ConstructionPlan, Step, Wrapping};
