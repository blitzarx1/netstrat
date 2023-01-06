use super::element::{Edge, Node};
use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct FrozenElements {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl FrozenElements {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> FrozenElements {
        FrozenElements { nodes, edges }
    }

    pub fn nodes(&self) -> &Vec<Node> {
        &self.nodes
    }

    pub fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }
}
