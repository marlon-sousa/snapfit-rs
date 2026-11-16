mod cycle_finder;
mod indexed_graph;

use cycle_finder::CycleFinder;
use indexed_graph::IndexedGraph;

use crate::domain::{DependencyIR, DomainError};

#[derive(Debug, Default, Clone)]
pub(crate) struct GraphEngine;

impl GraphEngine {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn topological_sort(
        &self,
        dependencies: &[DependencyIR],
    ) -> Result<Vec<DependencyIR>, DomainError> {
        IndexedGraph::try_from_dependencies(dependencies)?
            .topological_order()
            .map_err(DomainError::CycleDetected)
    }
}
