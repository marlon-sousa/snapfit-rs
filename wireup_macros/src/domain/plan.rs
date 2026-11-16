use crate::domain::ir::Lifecycle;
use derive_new::new;

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct InstantiationStep {
    #[new(into)]
    pub dep_name: String,
    #[new(into)]
    pub dep_type: String,
    #[new(into)]
    pub constructor_call: String,
    pub is_async: bool,
    pub lifecycle: Lifecycle,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct ConstructionPlan {
    #[new(into)]
    pub container_name: String,
    pub is_async: bool,
    #[new(into)]
    pub key_enum_name: String,
    #[new(default)]
    pub ordered_steps: Vec<InstantiationStep>,
    #[new(default)]
    pub parallel_batches: Vec<Vec<String>>,
}
