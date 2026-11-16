use derive_new::new;

use crate::domain::{
    Construction, ConstructionPlan, ContainerIR, DependencyIR, DomainError, InstantiationStep,
};

#[cfg(test)]
mod tests;

#[derive(new, Debug, Default, Clone)]
pub(crate) struct ExecutionPlanner;

impl ExecutionPlanner {
    pub(crate) fn build_plan(
        &self,
        container_ir: &ContainerIR,
        sorted_dependencies: Vec<DependencyIR>,
    ) -> Result<ConstructionPlan, DomainError> {
        let is_async = Self::is_plan_async(container_ir, &sorted_dependencies);

        let mut plan = ConstructionPlan::new(container_ir.name.clone(), is_async);
        plan.ordered_steps = sorted_dependencies
            .into_iter()
            .map(Self::build_step)
            .collect();
        Ok(plan)
    }

    fn is_plan_async(container_ir: &ContainerIR, dependencies: &[DependencyIR]) -> bool {
        container_ir.is_async || dependencies.iter().any(|dep| dep.is_async)
    }

    fn build_step(dep: DependencyIR) -> InstantiationStep {
        let construction = dep.custom_builder.map_or(
            Construction::Constructor(dep.constructor_name),
            Construction::Builder,
        );
        InstantiationStep::new(
            dep.name,
            dep.type_string,
            construction,
            dep.prerequisites,
            dep.is_async,
            dep.lifecycle,
        )
    }
}
