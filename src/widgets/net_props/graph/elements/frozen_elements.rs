use serde::{Deserialize, Serialize};

use super::ElementID;

#[derive(Debug, Hash, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct FrozenElements {
    nodes: Vec<ElementID>,
    edges: Vec<ElementID>,
}

impl FrozenElements {
    pub fn new(nodes: Vec<ElementID>, edges: Vec<ElementID>) -> FrozenElements {
        FrozenElements { nodes, edges }
    }

    pub fn nodes(&self) -> &Vec<ElementID> {
        &self.nodes
    }

    pub fn edges(&self) -> &Vec<ElementID> {
        &self.edges
    }
}
