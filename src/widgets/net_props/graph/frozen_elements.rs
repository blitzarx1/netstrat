use serde::{Deserialize, Serialize};

use super::elements::Elements;

#[derive(Debug, Hash, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct FrozenElements {
    pub nodes: Vec<(usize, String)>,
    pub edges: Vec<(usize, String)>,
}

impl FrozenElements {
    pub fn from_elements(elements: &Elements) -> Self {
        let mut nodes: Vec<(usize, String)> = elements
            .nodes()
            .iter()
            .map(|(k, v)| (k.index(), v.clone()))
            .collect();

        let mut edges: Vec<(usize, String)> = elements
            .edges()
            .iter()
            .map(|(k, v)| (k.index(), v.clone()))
            .collect();

        nodes.sort_by_key(|(k, _)| *k);
        edges.sort_by_key(|(k, _)| *k);

        Self { nodes, edges }
    }
}
