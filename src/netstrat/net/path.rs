use petgraph::graph::{EdgeIndex, NodeIndex};

#[derive(Clone)]
pub struct Path(NodeIndex, EdgeIndex, NodeIndex);

impl Path {
    pub fn new(start: NodeIndex, end: NodeIndex, edge: EdgeIndex) -> Self {
        Path(start, edge, end)
    }

    pub fn start(&self) -> NodeIndex {
        self.0
    }
    pub fn edge(&self) -> EdgeIndex {
        self.1
    }
    pub fn end(&self) -> NodeIndex {
        self.2
    }
}
