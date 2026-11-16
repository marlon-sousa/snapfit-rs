use crate::domain::{ConstructionPlan, ContainerIR, DependencyIR, DomainError, InstantiationStep};

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
        let is_async = container_ir.is_async || sorted_dependencies.iter().any(|dep| dep.is_async);
        let key_enum_name = format!("{}Key", container_ir.name);

        let ordered_steps = sorted_dependencies
            .into_iter()
            .map(|dep| Self::build_step(&container_ir.name, dep))
            .collect();

        let mut plan = ConstructionPlan::new(container_ir.name.clone(), is_async, key_enum_name);
        plan.ordered_steps = ordered_steps;
        Ok(plan)
    }

    fn build_step(
        container_name: &crate::domain::ContainerName,
        dep: DependencyIR,
    ) -> InstantiationStep {
        let args = dep
            .prerequisites
            .iter()
            .map(|prereq| format!("{prereq}.clone()"))
            .collect::<Vec<_>>()
            .join(", ");

        let constructor_call = if let Some(custom_builder) = &dep.custom_builder {
            format!("{container_name}::{custom_builder}({args})")
        } else {
            let target_type = Self::strip_arc_wrapper(dep.type_string.as_str());
            format!("{target_type}::{}({args})", dep.constructor_name)
        };

        InstantiationStep::new(
            dep.name,
            dep.type_string,
            constructor_call,
            dep.is_async,
            dep.lifecycle,
        )
    }

    fn strip_arc_wrapper(type_str: &str) -> &str {
        let trimmed = type_str.trim();
        for prefix in ["std::sync::Arc<", "alloc::sync::Arc<", "Arc<"] {
            if let Some(inner) = trimmed
                .strip_prefix(prefix)
                .and_then(|s| s.strip_suffix('>'))
            {
                return inner.trim();
            }
        }
        trimmed
    }
}
