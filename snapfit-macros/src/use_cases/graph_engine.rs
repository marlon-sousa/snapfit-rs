mod cycle_finder;
mod indexed_graph;

#[cfg(test)]
mod tests;

use derive_new::new;

use cycle_finder::CycleFinder;
use indexed_graph::IndexedGraph;

use crate::domain::{DependencyIR, DomainError};

#[derive(new, Debug, Default, Clone)]
pub(crate) struct GraphEngine;

impl GraphEngine {
    pub(crate) fn topological_sort(
        &self,
        dependencies: &[DependencyIR],
    ) -> Result<Vec<DependencyIR>, DomainError> {
        IndexedGraph::try_from_dependencies(dependencies)?
            .topological_order()
            .map_err(DomainError::CycleDetected)
    }
}
