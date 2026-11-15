pub mod errors;
pub mod ir;
pub mod plan;

#[cfg(test)]
mod tests;

pub use errors::DomainError;
pub use ir::{ContainerIR, DependencyIR, Lifecycle, TraitDispatch};
pub use plan::{ConstructionPlan, InstantiationStep};
