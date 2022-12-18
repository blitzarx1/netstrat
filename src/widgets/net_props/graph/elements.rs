use petgraph::graph::{EdgeIndex, NodeIndex};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;

use super::frozen_elements::FrozenElements;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
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
        self.frozen().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Elements {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let frozen = FrozenElements::deserialize(deserializer)?;

        let mut nodes = HashSet::with_capacity(frozen.nodes.len());
        frozen.nodes.iter().for_each(|n| {
            nodes.insert(NodeIndex::from(*n as u32));
        });

        let mut edges = HashSet::with_capacity(frozen.edges.len());
        frozen.edges.iter().for_each(|n| {
            edges.insert(EdgeIndex::from(*n as u32));
        });

        Ok(Elements::new(nodes, edges))
    }
}

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

        self.frozen = FrozenElements::from_elements(self);
    }

    pub fn add_node(&mut self, n: NodeIndex) -> bool {
        let res = self.nodes.insert(n);
        if res {
            self.frozen.nodes.push(n.index());
        };

        res
    }

    pub fn add_edge(&mut self, e: EdgeIndex) -> bool {
        let res = self.edges.insert(e);
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

#[cfg(test)]
mod test {
    use super::*;

    const SERIALIZED_DATA: &str = r#"{"nodes":[1,2,3],"edges":[4,5]}"#;

    fn elements() -> Elements {
        let mut nodes = HashSet::new();
        nodes.insert(NodeIndex::from(1));
        nodes.insert(NodeIndex::from(2));
        nodes.insert(NodeIndex::from(3));

        let mut edges = HashSet::new();
        edges.insert(EdgeIndex::from(4));
        edges.insert(EdgeIndex::from(5));

        Elements::new(nodes, edges)
    }

    #[test]
    fn test_serialize() {
        let elements = elements();

        let res = serde_json::to_string(&elements).unwrap();
        assert_eq!(res, SERIALIZED_DATA);
    }

    #[test]
    fn test_deserialize() {
        let res: Elements = serde_json::from_str(SERIALIZED_DATA).unwrap();

        assert_eq!(res, elements());
    }
}
