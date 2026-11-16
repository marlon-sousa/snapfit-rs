use crate::domain::ir::{ContainerName, DependencyName, Lifecycle, TypeName};
use derive_new::new;

#[cfg(test)]
mod tests;

/// How a dependency's value is produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Construction {
    /// An associated function on the dependency's own type, e.g. `Database::connect`.
    Constructor(String),
    /// An associated function on the container, e.g. `AppContainer::build_auth`.
    Builder(String),
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct InstantiationStep {
    #[new(into)]
    pub dep_name: DependencyName,
    #[new(into)]
    pub dep_type: TypeName,
    pub construction: Construction,
    pub prerequisites: Vec<DependencyName>,
    pub is_async: bool,
    pub lifecycle: Lifecycle,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct ConstructionPlan {
    #[new(into)]
    pub container_name: ContainerName,
    pub is_async: bool,
    #[new(default)]
    pub ordered_steps: Vec<InstantiationStep>,
}

impl ConstructionPlan {
    pub fn step(&self, name: &DependencyName) -> Option<&InstantiationStep> {
        self.ordered_steps
            .iter()
            .find(|step| &step.dep_name == name)
    }
}
