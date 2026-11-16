pub mod errors;
pub mod ir;
pub mod plan;

pub use errors::DomainError;
pub use ir::{
    ContainerIR, ContainerName, DependencyIR, DependencyName, Lifecycle, TraitDispatch, TypeName,
};
pub use plan::{ConstructionPlan, InstantiationStep};
