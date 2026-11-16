use derive_new::new;

use crate::domain::{ConstructionPlan, ContainerIR, DomainError};
use crate::use_cases::{ExecutionPlanner, GraphEngine};

#[derive(new, Debug, Default, Clone)]
pub(crate) struct DeriveContainerUseCase {
    #[new(default)]
    graph_engine: GraphEngine,
    #[new(default)]
    planner: ExecutionPlanner,
}

impl DeriveContainerUseCase {
    pub(crate) fn with_components(graph_engine: GraphEngine, planner: ExecutionPlanner) -> Self {
        Self {
            graph_engine,
            planner,
        }
    }

    pub(crate) fn execute(
        &self,
        container_ir: ContainerIR,
    ) -> Result<ConstructionPlan, DomainError> {
        let sorted_dependencies = self
            .graph_engine
            .topological_sort(&container_ir.dependencies)?;
        self.planner.build_plan(&container_ir, sorted_dependencies)
    }
}
