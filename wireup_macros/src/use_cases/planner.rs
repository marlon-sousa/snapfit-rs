use crate::domain::{
    ConstructionPlan, ContainerIR, ContainerName, DependencyIR, DependencyName, DomainError,
    InstantiationStep,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Default, Clone)]
pub(crate) struct ExecutionPlanner;

impl ExecutionPlanner {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn build_plan(
        &self,
        container_ir: &ContainerIR,
        sorted_dependencies: Vec<DependencyIR>,
    ) -> Result<ConstructionPlan, DomainError> {
        let is_async = Self::is_plan_async(container_ir, &sorted_dependencies);
        let key_enum_name = format!("{}Key", container_ir.name);

        let mut plan = ConstructionPlan::new(container_ir.name.clone(), is_async, key_enum_name);
        plan.ordered_steps = sorted_dependencies
            .into_iter()
            .map(|dep| Self::build_step(&container_ir.name, dep))
            .collect();
        Ok(plan)
    }

    fn is_plan_async(container_ir: &ContainerIR, dependencies: &[DependencyIR]) -> bool {
        container_ir.is_async || dependencies.iter().any(|dep| dep.is_async)
    }

    fn build_step(container_name: &ContainerName, dep: DependencyIR) -> InstantiationStep {
        let constructor_call = Self::format_constructor_call(container_name, &dep);
        InstantiationStep::new(
            dep.name,
            dep.type_string,
            constructor_call,
            dep.is_async,
            dep.lifecycle,
        )
    }

    fn format_constructor_call(container_name: &ContainerName, dep: &DependencyIR) -> String {
        let args = Self::format_cloned_args(&dep.prerequisites);

        if let Some(custom_builder) = &dep.custom_builder {
            return format!("{container_name}::{custom_builder}({args})");
        }

        let target_type = Self::strip_arc_wrapper(dep.type_string.as_str());
        format!("{target_type}::{}({args})", dep.constructor_name)
    }

    fn format_cloned_args(prerequisites: &[DependencyName]) -> String {
        prerequisites
            .iter()
            .map(|prereq| format!("{prereq}.clone()"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn strip_arc_wrapper(type_str: &str) -> &str {
        let trimmed = type_str.trim();
        ["std::sync::Arc<", "alloc::sync::Arc<", "Arc<"]
            .into_iter()
            .find_map(|prefix| trimmed.strip_prefix(prefix)?.strip_suffix('>'))
            .map_or(trimmed, str::trim)
    }
}
