use crate::domain::ir::Lifecycle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantiationStep {
    pub dep_name: String,
    pub dep_type: String,
    pub constructor_call: String,
    pub is_async: bool,
    pub lifecycle: Lifecycle,
}

impl InstantiationStep {
    pub fn new(
        dep_name: impl Into<String>,
        dep_type: impl Into<String>,
        constructor_call: impl Into<String>,
        is_async: bool,
        lifecycle: Lifecycle,
    ) -> Self {
        Self {
            dep_name: dep_name.into(),
            dep_type: dep_type.into(),
            constructor_call: constructor_call.into(),
            is_async,
            lifecycle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionPlan {
    pub container_name: String,
    pub is_async: bool,
    pub key_enum_name: String,
    pub ordered_steps: Vec<InstantiationStep>,
    pub parallel_batches: Vec<Vec<String>>,
}

impl ConstructionPlan {
    pub fn new(
        container_name: impl Into<String>,
        is_async: bool,
        key_enum_name: impl Into<String>,
    ) -> Self {
        Self {
            container_name: container_name.into(),
            is_async,
            key_enum_name: key_enum_name.into(),
            ordered_steps: Vec::new(),
            parallel_batches: Vec::new(),
        }
    }
}
