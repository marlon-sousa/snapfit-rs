use std::collections::{HashMap, HashSet};

use petgraph::algo::{tarjan_scc, toposort};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::domain::{DependencyIR, DependencyName, DomainError};

struct IndexedGraph<'a> {
    dependencies: &'a [DependencyIR],
    graph: DiGraph<usize, ()>,
    name_to_node: HashMap<DependencyName, NodeIndex>,
}

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
        let indexed = Self::build_indexed_graph(dependencies)?;

        match toposort(&indexed.graph, None) {
            Ok(sorted_nodes) => Ok(sorted_nodes
                .into_iter()
                .map(|node| indexed.dependencies[indexed.graph[node]].clone())
                .collect()),
            Err(_) => {
                let cycle_path = Self::extract_cycle_path(&indexed);
                Err(DomainError::CycleDetected(cycle_path))
            }
        }
    }

    fn build_indexed_graph(dependencies: &[DependencyIR]) -> Result<IndexedGraph<'_>, DomainError> {
        let mut graph = DiGraph::with_capacity(dependencies.len(), dependencies.len());
        let mut name_to_node = HashMap::with_capacity(dependencies.len());

        for (idx, dep) in dependencies.iter().enumerate() {
            let node = graph.add_node(idx);
            if name_to_node.insert(dep.name.clone(), node).is_some() {
                return Err(DomainError::DuplicateDependency(dep.name.clone()));
            }
        }

        for dep in dependencies {
            let missing: Vec<DependencyName> = dep
                .prerequisites
                .iter()
                .filter(|prereq| !name_to_node.contains_key(*prereq))
                .cloned()
                .collect();

            if !missing.is_empty() {
                return Err(DomainError::MissingPrerequisites {
                    dependency: dep.name.clone(),
                    missing,
                });
            }

            let dependent_node = name_to_node[&dep.name];
            for prereq_name in &dep.prerequisites {
                let prereq_node = name_to_node[prereq_name];
                // Edge prerequisite -> dependent ensures prerequisites are ordered first
                graph.add_edge(prereq_node, dependent_node, ());
            }
        }

        Ok(IndexedGraph {
            dependencies,
            graph,
            name_to_node,
        })
    }

    fn extract_cycle_path(indexed: &IndexedGraph<'_>) -> Vec<DependencyName> {
        for scc in tarjan_scc(&indexed.graph) {
            if let [node] = scc.as_slice() {
                if indexed.graph.contains_edge(*node, *node) {
                    let name = indexed.dependencies[indexed.graph[*node]].name.clone();
                    return vec![name.clone(), name];
                }
            } else if scc.len() > 1 {
                let scc_set: HashSet<NodeIndex> = scc.iter().copied().collect();
                let mut candidates = scc;
                candidates.sort_by_key(|&n| indexed.graph[n]);

                for start in candidates {
                    let mut visited = HashSet::new();
                    let mut path = Vec::new();
                    if Self::find_cycle_dfs(
                        indexed,
                        start,
                        start,
                        &scc_set,
                        &mut visited,
                        &mut path,
                    ) {
                        return path
                            .into_iter()
                            .map(|n| indexed.dependencies[indexed.graph[n]].name.clone())
                            .collect();
                    }
                }
            }
        }

        Vec::new()
    }

    fn find_cycle_dfs(
        indexed: &IndexedGraph<'_>,
        current: NodeIndex,
        target: NodeIndex,
        scc_set: &HashSet<NodeIndex>,
        visited: &mut HashSet<NodeIndex>,
        path: &mut Vec<NodeIndex>,
    ) -> bool {
        visited.insert(current);
        path.push(current);

        let dep = &indexed.dependencies[indexed.graph[current]];
        for prereq_name in &dep.prerequisites {
            let next = indexed.name_to_node[prereq_name];
            if !scc_set.contains(&next) {
                continue;
            }
            if next == target {
                path.push(target);
                return true;
            }
            if !visited.contains(&next)
                && Self::find_cycle_dfs(indexed, next, target, scc_set, visited, path)
            {
                return true;
            }
        }

        path.pop();
        visited.remove(&current);
        false
    }
}
