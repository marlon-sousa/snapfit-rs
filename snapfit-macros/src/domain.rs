//! Entity: the domain's data, free of any compiler type.

mod build;
mod declaration;
mod dispatch_plan;
mod dispatched_trait;
mod dispatched_type;
mod errors;
mod names;
mod plan;

pub(crate) use build::FeatureGate;
pub(crate) use declaration::{
    BuilderMethods, Construction, Constructor, ContainerDeclaration, DependencyDeclaration,
    Execution, Fallibility, Implementation, Introspection, Lifecycle, Prerequisite, TypeParameters,
    TypeShape,
};
pub(crate) use dispatch_plan::{DispatchPlan, ForwardedItem, Rebuilding};
pub(crate) use dispatched_trait::{
    Annotated, AssociatedTypeShape, DispatchArguments, DispatchAttribute, ItemPosition,
    MethodShape, PlacedItem, Presence, Receiver, ReturnShape, Safety, SelfBound, SelfUse,
    TraitDeclaration, TraitItem, TraitItemDeclaration,
};
pub(crate) use dispatched_type::{
    AttributePosition, Dispatch, DispatchExtension, DispatchedTypeDeclaration, DispatchedTypePlan,
    Variant, WrittenType,
};
pub(crate) use errors::{
    AsyncProblem, AttributeProblem, DispatchAttributeKind, DispatchProblem, DispatchSubject,
    DomainError, DomainErrors, Feature, InfallibleBuild, ItemProblem, StaticDispatchProblem,
    TraitObjectProblem,
};
pub(crate) use names::{
    ConstantName, ContainerName, DependencyName, FunctionName, MacroName, MethodName, SetterName,
    TraitName, TypeName,
};
pub(crate) use plan::{Call, ConstructionPlan, Step, Wrapping};
