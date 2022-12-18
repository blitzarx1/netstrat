use petgraph::graph::{EdgeIndex, NodeIndex};
use serde::{ser::SerializeStruct, Serialize};
use std::collections::HashSet;

use super::frozen_elements::FrozenElements;

#[derive(Default, Clone, Eq, PartialEq)]
pub struct Elements {
    nodes: HashSet<NodeIndex>,
    edges: HashSet<EdgeIndex>,
    frozen: FrozenElements,
}

impl Serialize for Elements {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let frozen = self.frozen();
        let mut s = serializer.serialize_struct("Elements", 2)?;
        s.serialize_field("nodes", &frozen.nodes)?;
        s.serialize_field("edges", &frozen.edges)?;
        s.end()
    }
}

// impl Deserialize for Elements {
//     fn deserialize<'de, D>(deserializer: D) -> Result<Self, dyn serde::de::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         todo!()
//     }
// }

impl Elements {
    pub fn new(nodes: HashSet<NodeIndex>, edges: HashSet<EdgeIndex>) -> Self {
        let mut res = Self {
            nodes,
            edges,
            frozen: Default::default(),
        };

        res.frozen = FrozenElements::from_elements(&res);

        res
    }

    pub fn union(&mut self, other: &Elements) {
        self.nodes = self.nodes.union(&other.nodes).cloned().collect();
        self.edges = self.edges.union(&other.edges).cloned().collect();

        self.frozen = FrozenElements::from_elements(&self);
    }

    pub fn add_node(&mut self, n: NodeIndex) -> bool {
        let res = self.nodes.insert(n.clone());
        if res {
            self.frozen.nodes.push(n.index());
        };

        res
    }

    pub fn add_edge(&mut self, e: EdgeIndex) -> bool {
        let res = self.edges.insert(e.clone());
        if res {
            self.frozen.edges.push(e.index());
        }

        res
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

    pub fn frozen(&self) -> &FrozenElements {
        &self.frozen
    }
}
