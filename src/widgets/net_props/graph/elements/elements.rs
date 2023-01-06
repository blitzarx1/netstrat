use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt::Debug, fmt::Display, ops::Sub, collections::HashSet};

use crate::widgets::history::Difference;

use super::{
    element::{Edge, Node},
    frozen_elements::FrozenElements,
};

const SIGN_NODES: &str = "🇳";
const SIGN_EDGES: &str = "🇪";
const offset: &str = "    ";

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Elements {
    nodes: HashSet<Node>,
    edges: HashSet<Edge>,
    frozen: FrozenElements,
}

// TODO: Don't use Display for such repr. Use caller formatter insted.
impl Display for Elements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "\n{}{}: {}\n{}{}: {}",
            SIGN_NODES,
            offset,
            self.frozen()
                .nodes()
                .iter()
                .map(|n| n.name().clone())
                .collect::<Vec<_>>()
                .join(", "),
            SIGN_EDGES,
            offset,
            self.frozen()
                .edges()
                .iter()
                .map(|e| e.name().clone())
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

        let mut nodes = frozen.nodes().iter().cloned().collect::<HashSet<_>>();
        let mut edges = frozen.edges().iter().cloned().collect::<HashSet<_>>();

        Ok(Elements::new(nodes, edges))
    }
}

impl Elements {
    pub fn new(nodes: HashSet<Node>, edges: HashSet<Edge>) -> Self {
        let mut res = Self {
            nodes,
            edges,
            frozen: Default::default(),
        };

        res.frozen = res.compute_frozen();

        res
    }

    fn compute_frozen(&self) -> FrozenElements {
        let mut nodes = self.nodes().iter().cloned().collect::<Vec<_>>();
        let mut edges = self.edges().iter().cloned().collect::<Vec<_>>();

        nodes.sort_by(|l, r| l.cmp(r));
        edges.sort_by(|l, r| l.cmp(r));

        FrozenElements { nodes, edges }
    }

    pub fn apply_difference(&self, diff: Difference) -> Elements {
        let res = self.union(&diff.plus);
        res.sub(&diff.minus)
    }

    /// computes difference which holds set of deleted and added elements
    pub fn compute_difference(&self, other: &Elements) -> Difference {
        let minus_nodes = self
            .nodes
            .iter()
            .filter(|n| !other.nodes.contains(n))
            .cloned()
            .collect::<HashSet<_>>();

        let minus_edges = self
            .edges()
            .iter()
            .filter(|e| !other.edges.contains(e))
            .cloned()
            .collect::<HashSet<_>>();

        let plus_nodes = self
            .nodes
            .iter()
            .filter(|n| !self.nodes.contains(n))
            .cloned()
            .collect::<HashSet<_>>();
        let plus_edges = self
            .edges()
            .iter()
            .filter(|e| !self.edges.contains(e))
            .cloned()
            .collect::<HashSet<_>>();

        Difference {
            plus: Elements::new(plus_nodes, plus_edges),
            minus: Elements::new(minus_nodes, minus_edges),
        }
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

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn nodes(&self) -> &HashSet<Node> {
        &self.nodes
    }

    pub fn edges(&self) -> &HashSet<Edge> {
        &self.edges
    }

    pub fn frozen(&self) -> &FrozenElements {
        &self.frozen
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // FIXME: fix test (change SERIALIZED_DATA)
    const SERIALIZED_DATA: &str =
        r#"{"nodes":[[1,"ini_1"],[2,"2"],[3,"fin_3"]],"edges":[[4,"ini_1->2"],[5,"2->fin_3"]]}"#;

    fn elements() -> Elements {
        let n1 = Node::new("ini_1".to_string());
        let n2 = Node::new("2".to_string());
        let n3 = Node::new("fin_3".to_string());
        let nodes = [n1, n2, n3].iter().cloned().collect::<HashSet<_>>();

        let e1 = Edge::new(&n1, &n2, 1.0);
        let e2 = Edge::new(&n2, &n3, 1.0);
        let mut edges = [e1, e2].iter().cloned().collect::<HashSet<_>>();

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
