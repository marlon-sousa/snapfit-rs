use crate::domain::ir::{ContainerName, DependencyName, Lifecycle, TypeName};
use derive_new::new;

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct InstantiationStep {
    #[new(into)]
    pub dep_name: DependencyName,
    #[new(into)]
    pub dep_type: TypeName,
    #[new(into)]
    pub constructor_call: String,
    pub is_async: bool,
    pub lifecycle: Lifecycle,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct ConstructionPlan {
    #[new(into)]
    pub container_name: ContainerName,
    pub is_async: bool,
    #[new(into)]
    pub key_enum_name: String,
    #[new(default)]
    pub ordered_steps: Vec<InstantiationStep>,
    #[new(default)]
    pub parallel_batches: Vec<Vec<DependencyName>>,
}
