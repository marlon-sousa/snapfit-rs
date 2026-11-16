use std::collections::{HashMap, HashSet};

use petgraph::algo::{tarjan_scc, toposort};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::domain::{DependencyIR, DependencyName, DomainError};

pub(super) struct IndexedGraph<'a> {
    dependencies: &'a [DependencyIR],
    graph: DiGraph<usize, ()>,
    name_to_node: HashMap<DependencyName, NodeIndex>,
}

impl<'a> IndexedGraph<'a> {
    pub(super) fn try_from_dependencies(
        dependencies: &'a [DependencyIR],
    ) -> Result<Self, DomainError> {
        let mut indexed = Self::with_capacity(dependencies);
        indexed.register_all_nodes()?;
        indexed.connect_all_prerequisites()?;
        Ok(indexed)
    }

    fn with_capacity(dependencies: &'a [DependencyIR]) -> Self {
        let count = dependencies.len();
        Self {
            dependencies,
            graph: DiGraph::with_capacity(count, count),
            name_to_node: HashMap::with_capacity(count),
        }
    }

    fn register_all_nodes(&mut self) -> Result<(), DomainError> {
        self.dependencies
            .iter()
            .enumerate()
            .try_for_each(|(idx, dep)| self.register_node(idx, dep))
    }

    fn register_node(&mut self, idx: usize, dep: &DependencyIR) -> Result<(), DomainError> {
        let node = self.graph.add_node(idx);
        self.name_to_node
            .insert(dep.name.clone(), node)
            .map_or(Ok(()), |_| {
                Err(DomainError::DuplicateDependency(dep.name.clone()))
            })
    }

    fn connect_all_prerequisites(&mut self) -> Result<(), DomainError> {
        self.dependencies
            .iter()
            .try_for_each(|dep| self.validate_and_connect(dep))
    }

    fn validate_and_connect(&mut self, dep: &DependencyIR) -> Result<(), DomainError> {
        self.ensure_all_prerequisites_exist(dep)?;
        self.add_prerequisite_edges(dep);
        Ok(())
    }

    fn ensure_all_prerequisites_exist(&self, dep: &DependencyIR) -> Result<(), DomainError> {
        let missing = self.missing_prerequisites_for(dep);
        missing
            .is_empty()
            .then_some(())
            .ok_or_else(|| DomainError::MissingPrerequisites {
                dependency: dep.name.clone(),
                missing,
                available: self.wireable_candidates_for(dep),
            })
    }

    fn wireable_candidates_for(&self, dep: &DependencyIR) -> Vec<DependencyName> {
        self.dependencies
            .iter()
            .filter(|candidate| candidate.name != dep.name)
            .map(|candidate| candidate.name.clone())
            .collect()
    }

    fn missing_prerequisites_for(&self, dep: &DependencyIR) -> Vec<DependencyName> {
        dep.prerequisites
            .iter()
            .filter(|prereq| !self.name_to_node.contains_key(*prereq))
            .cloned()
            .collect()
    }

    fn add_prerequisite_edges(&mut self, dep: &DependencyIR) {
        let dependent_node = self.name_to_node[&dep.name];
        dep.prerequisites
            .iter()
            .map(|prereq_name| self.name_to_node[prereq_name])
            .for_each(|prereq_node| {
                self.graph.add_edge(prereq_node, dependent_node, ());
            });
    }

    pub(super) fn topological_order(&self) -> Result<Vec<DependencyIR>, Vec<DependencyName>> {
        toposort(&self.graph, None)
            .map(|nodes| self.map_nodes_to_dependencies(nodes))
            .map_err(|_| self.first_cycle_path())
    }

    fn map_nodes_to_dependencies(&self, nodes: Vec<NodeIndex>) -> Vec<DependencyIR> {
        nodes
            .into_iter()
            .map(|node| self.dep_at(node).clone())
            .collect()
    }

    fn first_cycle_path(&self) -> Vec<DependencyName> {
        tarjan_scc(&self.graph)
            .into_iter()
            .find_map(|scc| self.cycle_in_component(scc))
            .unwrap_or_default()
    }

    fn cycle_in_component(&self, scc: Vec<NodeIndex>) -> Option<Vec<DependencyName>> {
        match scc.as_slice() {
            [node] => self.self_loop_cycle(*node),
            _ => self.multi_node_cycle(scc),
        }
    }

    fn self_loop_cycle(&self, node: NodeIndex) -> Option<Vec<DependencyName>> {
        self.graph.contains_edge(node, node).then(|| {
            let name = self.dep_name_at(node);
            vec![name.clone(), name]
        })
    }

    fn multi_node_cycle(&self, mut scc: Vec<NodeIndex>) -> Option<Vec<DependencyName>> {
        let component_nodes: HashSet<NodeIndex> = scc.iter().copied().collect();
        scc.sort_by_key(|&node| self.graph[node]);

        scc.into_iter()
            .find_map(|start| super::CycleFinder::new(self, &component_nodes).find_from(start))
    }

    pub(super) fn prerequisite_nodes_of(
        &self,
        node: NodeIndex,
    ) -> impl Iterator<Item = NodeIndex> + '_ {
        self.dep_at(node)
            .prerequisites
            .iter()
            .map(|name| self.name_to_node[name])
    }

    pub(super) fn dep_at(&self, node: NodeIndex) -> &DependencyIR {
        &self.dependencies[self.graph[node]]
    }

    pub(super) fn dep_name_at(&self, node: NodeIndex) -> DependencyName {
        self.dep_at(node).name.clone()
    }
}
