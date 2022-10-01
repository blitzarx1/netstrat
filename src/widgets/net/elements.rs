use petgraph::graph::{EdgeIndex, NodeIndex};
use std::collections::HashSet;

#[derive(Default)]
pub struct Elements {
    nodes: HashSet<NodeIndex>,
    edges: HashSet<EdgeIndex>,
}

impl Elements {
    pub fn new(nodes: HashSet<NodeIndex>, edges: HashSet<EdgeIndex>) -> Self {
        Self { nodes, edges }
    }

    pub fn union(&mut self, other: &Elements) {
        self.nodes = self.nodes.union(&other.nodes).cloned().collect();
        self.edges = self.edges.union(&other.edges).cloned().collect();
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn nodes(&self) -> HashSet<NodeIndex> {
        self.nodes.clone()
    }

    pub fn edges(&self) -> HashSet<EdgeIndex> {
        self.edges.clone()
    }
}
