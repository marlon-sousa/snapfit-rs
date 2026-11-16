use std::collections::HashSet;

use petgraph::graph::NodeIndex;

use crate::domain::DependencyName;
use crate::use_cases::graph_engine::indexed_graph::IndexedGraph;

pub(super) struct CycleFinder<'g, 'a> {
    indexed: &'g IndexedGraph<'a>,
    component_nodes: &'g HashSet<NodeIndex>,
    visited: HashSet<NodeIndex>,
    path: Vec<NodeIndex>,
}

impl<'g, 'a> CycleFinder<'g, 'a> {
    pub(super) fn new(
        indexed: &'g IndexedGraph<'a>,
        component_nodes: &'g HashSet<NodeIndex>,
    ) -> Self {
        Self {
            indexed,
            component_nodes,
            visited: HashSet::new(),
            path: Vec::new(),
        }
    }

    pub(super) fn find_from(mut self, start: NodeIndex) -> Option<Vec<DependencyName>> {
        self.search(start, start).then(|| self.into_cycle_names())
    }

    fn into_cycle_names(self) -> Vec<DependencyName> {
        self.path
            .into_iter()
            .map(|node| self.indexed.dep_name_at(node))
            .collect()
    }

    /// Depth-first search for a path back to `target`. Nodes stay visited
    /// after a dead end: if `target` was not reachable from a node once, it
    /// will not be on a later branch either, so the search stays linear.
    fn search(&mut self, current: NodeIndex, target: NodeIndex) -> bool {
        self.enter(current);
        let found = self
            .indexed
            .prerequisite_nodes_of(current)
            .any(|next| self.step_to(next, target));
        if !found {
            self.path.pop();
        }
        found
    }

    fn step_to(&mut self, next: NodeIndex, target: NodeIndex) -> bool {
        if !self.component_nodes.contains(&next) {
            return false;
        }
        if next == target {
            self.path.push(target);
            return true;
        }
        !self.visited.contains(&next) && self.search(next, target)
    }

    fn enter(&mut self, node: NodeIndex) {
        self.visited.insert(node);
        self.path.push(node);
    }
}
