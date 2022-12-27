use petgraph::graph::{EdgeIndex, NodeIndex};
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, fmt::Display};

use crate::widgets::history::Difference;

use super::frozen_elements::FrozenElements;

const SIGN_NODES: &str = "🇳";
const SIGN_EDGES: &str = "🇪";

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Elements {
    nodes: HashMap<NodeIndex, String>,
    edges: HashMap<EdgeIndex, String>,
    frozen: FrozenElements,
}

impl Display for Elements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "\n    {}: {}\n    {}: {}",
            SIGN_NODES,
            self.frozen()
                .nodes
                .iter()
                .map(|(_, repr)| repr.clone())
                .collect::<Vec<_>>()
                .join(", "),
            SIGN_EDGES,
            self.frozen()
                .edges
                .iter()
                .map(|(_, repr)| repr.clone())
                .collect::<Vec<_>>()
                .join(", "),
        ))
    }
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

        let mut nodes = HashMap::with_capacity(frozen.nodes.len());
        frozen.nodes.iter().for_each(|(k, v)| {
            nodes.insert(NodeIndex::from(*k as u32), v.clone());
        });

        let mut edges = HashMap::with_capacity(frozen.edges.len());
        frozen.edges.iter().for_each(|(k, v)| {
            edges.insert(EdgeIndex::from(*k as u32), v.clone());
        });

        Ok(Elements::new(nodes, edges))
    }
}

impl Elements {
    pub fn new(nodes: HashMap<NodeIndex, String>, edges: HashMap<EdgeIndex, String>) -> Self {
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
    pub fn compute_difference(&self, other: &Elements) -> Difference {
        let mut minus_nodes = HashMap::new();
        self.nodes.iter().for_each(|(k, v)| {
            if !other.nodes.contains_key(k) {
                minus_nodes.insert(*k, v.to_string());
            };
        });
        let mut minus_edges = HashMap::new();
        self.edges.iter().for_each(|(k, v)| {
            if !other.edges.contains_key(k) {
                minus_edges.insert(*k, v.to_string());
            };
        });

        let mut plus_nodes = HashMap::new();
        other.nodes.iter().for_each(|(k, v)| {
            if !self.nodes.contains_key(k) {
                plus_nodes.insert(*k, v.to_string());
            };
        });
        let mut plus_edges = HashMap::new();
        other.edges.iter().for_each(|(k, v)| {
            if !self.edges.contains_key(k) {
                plus_edges.insert(*k, v.to_string());
            };
        });

        if plus_nodes.is_empty()
            && plus_edges.is_empty()
            && minus_nodes.is_empty()
            && minus_edges.is_empty()
        {
            return Default::default();
        };

        Difference {
            plus: Elements::new(plus_nodes, plus_edges),
            minus: Elements::new(minus_nodes, minus_edges),
        }
    }

    pub fn union(&self, other: &Elements) -> Elements {
        let mut nodes = self.nodes.clone();
        other.nodes.iter().for_each(|(k, v)| {
            nodes.insert(*k, v.clone());
        });

        let mut edges = self.edges.clone();
        other.edges.iter().for_each(|(k, v)| {
            edges.insert(*k, v.clone());
        });

        Elements::new(nodes, edges)
    }

    pub fn sub(&self, other: &Elements) -> Elements {
        let mut nodes = HashMap::new();
        self.nodes.iter().for_each(|(k, v)| {
            if other.nodes.contains_key(k) {
                return;
            }
            nodes.insert(*k, v.clone());
        });

        let mut edges = HashMap::new();
        self.edges.iter().for_each(|(k, v)| {
            if other.edges.contains_key(k) {
                return;
            }
            edges.insert(*k, v.clone());
        });

        Elements::new(nodes, edges)
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn nodes(&self) -> HashMap<NodeIndex, String> {
        self.nodes.clone()
    }

    pub fn edges(&self) -> HashMap<EdgeIndex, String> {
        self.edges.clone()
    }

    pub fn frozen(&self) -> &FrozenElements {
        &self.frozen
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SERIALIZED_DATA: &str =
        r#"{"nodes":[[1,"ini_1"],[2,"2"],[3,"fin_3"]],"edges":[[4,"ini_1->2"],[5,"2->fin_3"]]}"#;

    fn elements() -> Elements {
        let mut nodes = HashMap::new();
        nodes.insert(NodeIndex::from(1), "ini_1".to_string());
        nodes.insert(NodeIndex::from(2), "2".to_string());
        nodes.insert(NodeIndex::from(3), "fin_3".to_string());

        let mut edges = HashMap::new();
        edges.insert(EdgeIndex::from(4), "ini_1->2".to_string());
        edges.insert(EdgeIndex::from(5), "2->fin_3".to_string());

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
