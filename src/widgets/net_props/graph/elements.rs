use petgraph::graph::{EdgeIndex, NodeIndex};
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashSet, hash::Hash, ops::Sub};

use crate::widgets::{history::Difference, StepDifference};

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

    pub fn apply_difference(&self, diff: Difference) -> Elements {
        let res = self.union(&diff.plus);
        res.sub(&diff.minus)
    }

    /// computes difference which holds set of deleted and added elements
    pub fn difference(&self, other: &Elements) -> Option<Difference> {
        let mut minus_nodes = HashSet::new();
        self.nodes.iter().for_each(|n| {
            if !other.nodes.contains(n) {
                minus_nodes.insert(*n);
            };
        });
        let mut minus_edges = HashSet::new();
        self.edges.iter().for_each(|e| {
            if !other.edges.contains(e) {
                minus_edges.insert(*e);
            };
        });

        let mut plus_nodes = HashSet::new();
        other.nodes.iter().for_each(|n| {
            if !self.nodes.contains(n) {
                plus_nodes.insert(*n);
            };
        });
        let mut plus_edges = HashSet::new();
        other.edges.iter().for_each(|n| {
            if !self.edges.contains(n) {
                plus_edges.insert(*n);
            };
        });

        if plus_nodes.is_empty()
            && plus_edges.is_empty()
            && minus_nodes.is_empty()
            && minus_edges.is_empty()
        {
            return None;
        };

        return Some(Difference {
            plus: Elements::new(plus_nodes, plus_edges),
            minus: Elements::new(minus_nodes, minus_edges),
        });
    }

    pub fn union(&self, other: &Elements) -> Elements {
        let nodes = self.nodes.union(&other.nodes).cloned().collect();
        let edges = self.edges.union(&other.edges).cloned().collect();

        Elements::new(nodes, edges)
    }

    pub fn sub(&self, other: &Elements) -> Elements {
        let nodes = self.nodes.sub(&other.nodes);
        let edges = self.edges.sub(&other.edges);

        Elements::new(nodes, edges)
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
