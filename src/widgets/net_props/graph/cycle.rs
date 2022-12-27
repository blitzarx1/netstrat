use std::collections::HashSet;

use petgraph::stable_graph::{EdgeIndex, NodeIndex};

use super::path::Path;

#[derive(Clone)]
pub struct Cycle(Vec<Path>);

impl Cycle {
    pub fn new() -> Self {
        Cycle(vec![])
    }

    pub fn add_path(&mut self, p: Path) {
        self.0.push(p)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn nodes_and_edges(&self) -> (HashSet<NodeIndex>, HashSet<EdgeIndex>) {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        self.0.iter().for_each(|p| {
            nodes.insert(p.start());
            nodes.insert(p.end());
            edges.insert(p.edge());
        });

        (nodes, edges)
    }
}
