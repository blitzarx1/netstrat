use serde::{Deserialize, Serialize};

use super::elements::Elements;

#[derive(Hash, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct FrozenElements {
    pub nodes: Vec<usize>,
    pub edges: Vec<usize>,
}

impl FrozenElements {
    pub fn from_elements(elements: &Elements) -> Self {
        let mut nodes: Vec<usize> = elements
            .nodes()
            .iter()
            .cloned()
            .map(|el| el.index())
            .collect();
        nodes.sort();

        let mut edges: Vec<usize> = elements
            .edges()
            .iter()
            .cloned()
            .map(|el| el.index())
            .collect();
        edges.sort();

        Self { nodes, edges }
    }
}
